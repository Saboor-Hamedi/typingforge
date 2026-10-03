# Suggestions — TypingForge ("Velotype") Code Review

Deep audit of `A:\rust\typingforge\src`. Items are grouped by category,
each with file:line references. Severity: 🔴 critical, 🟠 important, 🟡 minor.

---

## 1. Correctness Bugs

1. 🔴 **FTS5 rowids never correlated with base tables** — `db/migrations.rs:103-106,134-137,164-167`, `db/queries.rs:279-282,846-849,859-862`.
   `INSERT INTO texts_fts (title, content)` / `INSERT INTO passages_fts (...)` omit `rowid`,
   so FTS rowids auto-assign and drift out of sync with `texts.id`/`passages.id` after any
   delete. But `delete_user` (`db/queries.rs:91`), `delete_passage` (`db/queries.rs:809,813`),
   `update_passage` (`db/queries.rs:785,795`), and `truncate_database` delete/update by rowid.
   Result: wrong FTS rows deleted/updated, stale index entries, ghost search hits.
   `insert_passage` (`db/queries.rs:412-415`) *does* set `rowid` — inconsistent with `insert_text`.
   Fix: always insert FTS rowid explicitly and mirror base rowid on update/delete, or use
   external-content FTS5 tables with triggers.

2. 🔴 **`search_texts` joins FTS to base rows on title+content instead of rowid** —
   `db/queries.rs:299-305`. Duplicate titles/contents collapse results, miss rows, pair a
   match with the wrong document, and `ORDER BY rank` references the wrong table → arbitrary
   ranking. Fix: `JOIN texts t ON t.id = f.rowid`.

3. 🔴 **Error shake flag inverted/dead** — `typing/engine.rs:422-426`: `on_mistake` sets
   `self.has_error_shake = false;` (never `true`). Velocity-graph error ticks
   (`typing/engine.rs:240`) and ScreenShake-on-error never fire. Also `shake.trigger()` is
   never called anywhere and `play_error()` (`audio.rs:164`) is never called — dead features.

4. 🟠 **"Back to Typing" on Results restarts the game** — `ui/header.rs:297-301`: the back
   button sets `*on_play_again = true;`. Both buttons reload a fresh passage; the label lies.

5. 🟠 **Skipped chars on Space press aren't logged as keystrokes** — `typing/engine.rs:302-322`:
   unfinished chars are marked `Incorrect` and counted in `incorrect_keystrokes`, but no
   `KeystrokeRecord` is pushed, so persisted keystroke logs undercount errors vs. stats.

6. 🟠 **Typing a typo stores `typed` but never renders it** — `ui/typing_area.rs:169` renders
   `ch.expected` always; `DisplayChar.typed` is dead. Typos show expected char in red, so the
   user never sees what they actually typed. Combined with caret advancing on error, feedback
   is confusing.

7. 🟠 **Editor "Apply" double-inserts into `passages` AND `texts`** —
   `ui/panels/editor.rs:143-148`: every apply writes duplicate rows to both tables (and both
   FTS indexes). Palette dedupe by text hides this (`db/queries.rs:643`) but tables grow.

8. 🟠 **Timed mode uses `word_target` (25/40 *seconds*) to pick a passage with 25/40 *words*** —
   `app.rs:120-125`. Fragile coupling; a 40s passage is treated as 40 words.

9. 🟠 **`is_new_pb` tie logic isn't a total order** — `app.rs:165-182`: in the accuracy-epsilon
   band it requires strictly greater consistency; equal-consistency near-ties flip-flop on the
   `+0.05` epsilon. Also `net_wpm > 5.0` means a slow first run is never a PB.

10. 🟡 **Timed-mode first keystroke always gets latency from t=0** — `typing/engine.rs:291-295`:
    `last_keystroke_time` is updated before computing `latency_ms` from its stale value.

11. 🟡 **`LiveMetrics::update_frame` EMA is frame-rate dependent** — `typing/metrics.rs:71`:
    `ema_alpha * (dt/0.016)` clamps to 1.0 above 62.5 ms frames; not dt-corrected
    (`1 - exp(-dt/tau)`), so WPM smoothing differs at 60 vs 144 Hz.

12. 🟡 **`is_version_newer` not semver-robust** — `ui/updater.rs:513-533`: prerelease tags
    (`v1.2.0-rc1`) fail to parse and compare as "not newer".

13. 🟡 **`search_passages` LIKE wildcards not escaped** — `db/queries.rs:695`: `%`/`_` in user
    input act as wildcards ("100%" matches everything containing "100"). Parameterized so no
    injection, but wrong match semantics.

14. 🟡 **`escape_fts5_query` strips punctuation; empty result silently returns recent texts** —
    `utils/text.rs:66-80`: searching "C++" or "!!!" yields `don*`-style tokens or the recent-
    list fallback — surprising.

15. 🟡 **In `Words` mode the passage may not match `word_target`** — `db/queries.rs:458-475`
    falls back to *any* closest word_count, so a 25-word target can return a 200-word passage
    and the HUD/target diverge.

16. 🟡 **Backspace never restores streak state and Ctrl+Backspace re-counts stats** —
    `typing/engine.rs:387-419`: backspaced typos keep their incorrect counts (by design for
    accuracy) but retyped chars overwrite statuses, so both attempts count; inconsistent.

17. 🟡 **`elapsed_time`/`window_duration` edge** — `typing/engine.rs:206`: instant WPM uses the
    window's oldest keystroke; two keys 1.15 s apart still yield a nonzero burst reading.

---

## 2. Race Conditions / Threading

18. 🔴 **Updater spawns a download thread every frame while in `UpdateAvailable`** —
    `ui/updater.rs:429-444`: `render_button`'s `UpdateAvailable` arm calls
    `self.start_download()` each frame. Status only flips to `Downloading` once the spawned
    thread runs, so egui (>60 fps, and it calls `request_repaint()`) can spawn tens of curl
    processes racing on the same temp file. Fix: set status to `Downloading` synchronously in
    `start_download`, and never mutate/spawn from the render path.

19. 🟠 **Silent auto-download of updates on startup** — `ui/panels/settings/mod.rs:50-53`:
    `SettingsPanel::new()` calls `check_for_updates(true)` → downloads and stages an installer
    without consent, then `apply_and_restart` can `process::exit(0)`. Fix: check silently,
    prompt before download.

20. 🟠 **Mutex poisoning cascades across the app** — `db/connection.rs:48` and many
    `lock().unwrap()` in `ui/updater.rs`, `profile.rs:462,585,614`. One panic in a download/
    import thread while holding the lock kills every later UI frame. Fix: handle `PoisonError`
    or use `parking_lot::Mutex`.

21. 🟠 **Import/export holds the single `Arc<Mutex<Connection>>` while the UI also needs it** —
    `profile.rs:587-596,616-625`. Long JSON streaming blocks all UI-side DB work (fuzzy
    palette search, passage loading). Fix: use a separate read connection or run entirely off
    the shared writer.

22. 🟡 **No Cancel button actually cancels** — `profile.rs:44,61,582,584,611,613`:
    `sync_cancel` is declared, reset to false, and passed to export/import (which do check it,
    `typing_backup.rs:52,103`, `typing_import.rs:175`), but nothing in the UI ever stores
    `true`. The advertised cancel path is dead.

23. 🟡 **`restart_game` / DB open errors swallowed with only `eprintln!`** — silent in-memory
    DB fallback (`app.rs:43-46`) means all results vanish on exit with no UI notice.

---

## 3. Performance

24. 🟠 **Typing view re-wraps and re-measures every character every frame** —
    `ui/typing_area.rs:23-26,36-138,141-178`: lays out every char individually, builds the full
    paint list each frame, calls `ui.fonts(layout_no_wrap("M"))` per frame. O(chars) forever.

25. 🟠 **Continuous `request_repaint()` even when idle** — `app.rs:754` (+ updater's
    `request_repaint()`): 60+ fps repaint burns CPU/battery; no idle sleep.

26. 🟠 **Velocity history is unbounded; graph resampled every frame** — `typing/engine.rs:234-243`
    pushes ~6.7 pts/s for the whole session; `ui/velocity_graph.rs` resamples Catmull-Rom each
    frame with no cap.

27. 🟡 **`words` iterated 4× per frame in engine update** — `typing/engine.rs:179-185`: decays
    `pop_anim` for every char every frame even when nothing animates.

28. 🟡 **`get_random_passage_for_words`/`get_random_custom_passage` in `reset()` waste work** —
    `typing/engine.rs:103-138` generates 120 random words, then `load_passage_text` overwrites
    them for every custom passage load.

29. 🟡 **Profile tab fetches 100 session rows every frame** — `profile.rs:145-158`: should be
    a SQL aggregate (`COUNT`, `AVG`, `MAX`).

30. 🟡 **No voice limiting on click audio** — `audio.rs:104-161`: each keystroke spawns a new
    `FnSource`; fast typing queues dozens of overlapping 30-60 ms sources.

31. 🟡 **`main.rs` publish subcommand falls through to launching the UI on failure** —
    `main.rs:8-22`: a desktop app shelling out to `cargo` from PATH; on failure the app
    *still launches*. Pick one behavior.

---

## 4. UI/UX

32. 🔴 **Username truncated by byte index can panic (release `panic="abort"` → crash)** —
    `ui/header.rs:357-361`. `LocalAuth::register` accepts any non-empty username
    (`auth/local.rs:56`), so `Saboor🚀` crashes the header chip. Fix: `chars().take(n)`.

33. 🟠 **Resize hit-zones overlap header buttons** — `app.rs:439-480`: the 8 px top border
    overlaps the header's top 8 px; hovering the top edge of close/min/max sets `ResizeNorth`
    and a press starts a resize instead of the click.

34. 🟠 **Esc is overloaded with three meanings** — `app.rs:297-306`: Esc on Typing→Settings,
    Settings→Typing, Results→restart. Users expect Esc to close the palette/menu. Footer hints
    contradict actual behavior (`ui/panels/*` footers, Results footer "tab+enter play again"
    while plain Enter/Space also restart, `app.rs:288`).

35. 🟠 **Window buttons fade to 0.28 alpha while typing** — `app.rs:429-431`: close/min/max
    become hard to see exactly in flow state.

36. 🟠 **Header drag region covers mode tabs/buttons** — `ui/header.rs:43-49`: `click_and_drag`
    over the whole bar; drags starting on a tab issue `ViewportCommand::StartDrag`.

37. 🟡 **Custom Texts tab says "strict 25/40 word limits" but defaults to 20** —
    `custom_texts.rs:99` vs `custom_texts.rs:66-67`.

38. 🟡 **`unreachable!()` panic path in settings match** — `settings/mod.rs:178`: with
    `panic="abort"` this is a hard crash if ever reached.

39. 🟡 **Truncated-by-byte FTS search and raw LIKE on title+content** — searching is
    case-folded via `LOWER()` but not trimmed/normalized consistently across tables.

40. 🟡 **Modal fuzzy palette isn't modal** — it opens over Results/Settings; selecting a
    passage silently switches to Typing (`app.rs:258-260`).

41. 🟡 **Footer abbreviations "net:", "acc:" with no tooltips** — `app.rs:676-718`.

42. 🟡 **"high scores" wording stale; table is `personal_bests`** — `profile.rs:646`.

43. 🟡 **Weak password policy** — `auth/local.rs:56-64`: min length 3, no strength rules;
    username uniqueness TOCTOU mapped to a generic DB error (`auth/local.rs`).

44. 🟡 **Minimum window 860×600 with fixed sidebars** — settings content clamps to
    `max(360.0)` (`settings/mod.rs:104-108`) → overlap/clipping at small sizes.

45. 🟡 **No accessibility**: icon-only buttons have no screen-reader names, no reduced-motion
    setting despite heavy animation.

46. 🟡 **"Velotype"/"Tylotype" title vs `forgetyping` crate name** — `Cargo.toml`,
    `app.rs` window title, `ui/header.rs`. Pick one.

---

## 5. Dead Code / Duplication / Naming

47. 🔴 **Duplicate `AppConfig`/`StorageManager` module** — `storage.rs:1-97` vs
    `data/loader.rs:9-79`: two different config structs, two dirs (`com/Velotype/velotype` vs
    `com/velotype/velotype`, `data/paths.rs:4-6`), one writing `settings.json` the other
    `config.json`. `ui/settings.rs` (324 lines, `SettingsScreen`) is re-exported
    (`ui/mod.rs:19`) but never used. Delete both or wire one in.

48. 🟠 **Dead modules**: `ui/results.rs` (`ResultsScreen` never constructed; live one is
    `ui/panels/results_view.rs`), `game/engine.rs:1` 1-line re-export shim,
    `TextGenerator::random_quote()` unused, `play_error()` unused (see #3),
    `TypingAreaWidget::draw`'s `_is_cur_word`/`scroll_y` hardcoded 0 (`ui/typing_area.rs:131,141`).

49. 🟠 **Negative-id sentinel overloads `DbPassage.id`** — `db/queries.rs:631-638,747-754,789,812`:
    texts are smuggled in as passages with `id = -id`. Fragile; any consumer treating
    `passage.id` as a PK misbehaves. Replace with an enum or union type.

50. 🟡 **Dead fields**: `VelocityPoint.is_error` (always false, #3),
    `DisplayChar.shake_anim` (`game/text.rs:17`), `KeystrokeRecord.latency_ms` unused in
    analytics, `AppConfig.high_scores` HashMap never read, `include_punctuation`/
    `include_numbers` never set by UI, `InternalMetrics.internal_raw_wpm` partially used.

51. 🟡 **Stringly-typed columns**: `DbText.source` ("seed"|"user_paste"|"user_edit") and
    `DbPassage.category` should be enums.

52. 🟡 **Hardcoded modes 25/40 only** — no 60 s/120-word options (`typing/engine.rs:12-22`).

53. 🟡 **`SEED_PASSAGES` vs `CURATED_SEED_PASSAGES` inconsistency** — `data/loader.rs:82-106`
    vs `db/migrations.rs:5-58`; one seeds texts, the other passages.

54. 🟡 **No crate docs, all modules `pub`** — `lib.rs`: internal APIs exposed publicly.

55. 🟡 **`Cargo.toml` missing metadata** — no `license`, `repository`, `homepage`; `LICENSE`
    exists at `src/LICENSE`. `panic="abort"` in release + the hazards above → audit or unwind.

56. 🟡 **`src/target` build artifacts inside the second repo** — outer repo root is
    `typingforge/` but the crate (and its own `.git`) lives in `typingforge/src/`; verify
    `.gitignore` covers `target/`.

---

## 6. Missing Error Handling

57. 🟠 **`ConfigLoader::save` ignores write errors** — `data/loader.rs:73-78`: settings
    silently don't persist on read-only dirs.

58. 🟠 **`app_data_dir()` panics if OS dirs unavailable** — `data/paths.rs:10-12`:
    sandboxed/locked-down users crash at startup instead of a temp fallback.

59. 🟠 **All DB query errors swallowed** — `app.rs:107,115-131`, `editor.rs:144-145`,
    `profile.rs`: passage-load failure silently falls back to random words; editor
    `insert_*` errors invisible (user thinks it saved); PB-save failure swallowed
    (`app.rs:238`); FTS insert/update failures (`let _ = tx.execute(...)`,
    `db/queries.rs:412-415,784-799`) cause silent index drift.

60. 🟡 **Guest PB sentinel `user_id = 0` while sessions store `NULL`** — `app.rs:156`,
    `db/models.rs:50`: inconsistent user identification between tables.

61. 🟡 **`DbQueries::delete_user` FTS delete correlated to `texts.id` via rowid bug (#1)** —
    deleting a user leaves their texts orphaned in FTS, leaking deleted content into ranking.

62. 🟡 **Import has no size guard** — importing a huge JSON can OOM/throw; error surfaces as a
    string via `SyncOp::Failed` (OK) but no streaming limit.

---

## 7. Security / Robustness

63. 🔴 **Updater downloads an `.exe` with no checksum/signature verification and executes it** —
    `ui/updater.rs:229-359`. MITM or a compromised GitHub account → RCE. Fix: verify a SHA256
    asset from the release, then install.

64. 🟠 **Auto-download on startup (#19) + no verification (#63)** is the biggest real-world risk.

65. 🟡 **Plaintext password not zeroized** — `auth/local.rs`: `authenticate(&str)` keeps the
    plaintext in memory. Minor for local-only storage.

66. 🟡 **Hardcoded GitHub API URL** — `ui/updater.rs:107`: a fork/rename silently checks the
    wrong repo; releases rely on tag strings since `published_at` isn't deserialized.

---

## Top 10 to Fix First

1. FTS5 rowid/data correlation across all insert/update/delete/truncate paths (#1, #2, #61).
2. Updater thread storm from render path + missing checksum verification (#18, #63, #64).
3. `has_error_shake` never set; `shake.trigger()`/`play_error()` never called (#3).
4. Header `truncate(7)` byte-split panic; release is `panic="abort"` (#32, #55).
5. Resize hit-zones overlapping header buttons (#33).
6. Silent in-memory DB fallback losing user data (#23, #58, #59).
7. Dead/duplicate modules: `storage.rs`, `ui/settings.rs`, `ui/results.rs`, negative-id
   sentinel (#47, #48, #49).
8. Continuous repaint + full per-frame text layout = battery/CPU drain (#24, #25, #26).
9. Editor double-insert into `passages`+`texts` (#7); FTS insert errors swallowed (#59).
10. Esc overload + footer hints contradicting behavior (#34); "Back to Typing" restarts (#4).
