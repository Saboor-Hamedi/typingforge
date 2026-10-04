# Suggestions — TypingForge ("Velotype") Code Review & Fix Status

Audit of the crate at `A:\rust\typingforge\src` (package `forgetyping`, v0.1.17).

A large part of this list had already been fixed in the tree before this pass
(commit `a77d0ca` "most of the issue have solved"). This document records the
**verified current state** and the fixes applied in this pass.

Legend: ✅ fixed · 🟡 remaining / accepted · ⚪ already resolved before this pass

---

## Fixes applied in this pass

- ✅ **Password policy** — `auth/local.rs`: minimum password length is 3
  (explicit product choice), now measured with `chars().count()` instead of bytes;
  hint updated in `profile.rs` to "At least 3 characters".
- ✅ **No silent auto-download of updates** — `ui/panels/settings/mod.rs`:
  startup check now calls `check_for_updates(false)`, so an available update is
  only surfaced; downloading requires an explicit click.
- ✅ **Mutex-poison panics removed** — replaced `lock().unwrap()` with
  `lock().unwrap_or_else(|p| p.into_inner())` in `profile.rs` (6 sites),
  `custom_texts.rs` (3 sites) and `updates.rs` (1 site), matching the pattern
  already used in `db/connection.rs` and `ui/updater.rs`.
- ✅ **NaN-unsafe float comparison** — `ui/velocity_graph.rs`: the hover
  nearest-point search now uses `f32::total_cmp` instead of
  `partial_cmp(..).unwrap()`.
- ✅ **Custom-passages target inconsistency** — `ui/panels/settings/custom_texts.rs`
  offered 20/40 while the engine only supports 25/40 (`WordCountTarget`,
  `TimedDuration`). Aligned the word/time buttons, defaults, edit-detection and
  the time-mode word cap to **25/40**; fixed the time cap that referenced
  never-selectable 15/30 values.
- ✅ **Results "Back to Typing" actually restarts** — `ui/panels/results_view.rs`
  + `app.rs`: the in-page Back button previously set the restart flag (same as
  "Play Again"). It now navigates back to the Typing screen without reloading
  (new `on_back` flag).
- ✅ **Dead code removed** — `game/text.rs`: dropped the never-called
  `TextGenerator::random_quote()` and the never-read `DisplayChar.shake_anim`
  field.

Verification: `cargo check` clean, `cargo test --lib` → 16 passed / 0 failed.
(`cargo clippy` is unusable in this environment: cached dependencies were built
by an incompatible rustc.)

---

## Already resolved before this pass (verified)

These were reported earlier but are correct in the current tree:

- ⚪ FTS5 tables now store an explicit correlated `rowid` on every insert
  (`insert_text`, `insert_passage`, migrations, `truncate_database`,
  `insert_passages_batch`), and `search_texts` joins on `f.rowid = t.id`.
  Updates/deletes propagate FTS errors instead of ignoring them.
- ⚪ `has_error_shake` is set `true` in `on_mistake`; `shake.trigger()`,
  `audio.play_error()` and `audio.play_click()` are wired in `app.rs`.
- ⚪ Screen shake offset, particle and repaint gating implemented;
  `needs_continuous_repaint` avoids an unconditional repaint loop.
- ⚪ Velocity history is capped (1200 points) with downsampling.
- ⚪ Username chip truncation uses `chars().take(7)` (no UTF-8 byte-split panic).
- ⚪ Window-control hit area is excluded from resize zones; window buttons stay
  at full opacity; header drag region no longer covers the controls; typing
  chrome alpha raised from 0.28 to 0.55.
- ⚪ Duplicate/dead modules removed: `storage.rs`, `ui/settings.rs`,
  `ui/results.rs`, `game/engine.rs`.
- ⚪ `data/paths.rs` no longer panics (falls back to temp dir); config save
  returns and logs errors.
- ⚪ `search_passages` escapes `%`/`_`; `escape_fts5_query` returns no matches
  for punctuation-only queries; `get_user_stats_summary` is a single SQL
  aggregate instead of fetching 100 rows per frame.
- ⚪ Editor no longer double-inserts into both `passages` and `texts`; insert
  errors are surfaced in the UI.
- ⚪ Backup/import cancel flag is wired to a Cancel button and checked in the
  streaming loops; import enforces a 100 MB size guard.
- ⚪ Audio click has voice limiting (max 6 voices, 14 ms minimum interval).
- ⚪ `Cargo.toml` has `license`, `repository`, `homepage`.
- ⚪ Settings `Preview` tab is handled (no `unreachable!()` panic path).

---

## Remaining / accepted

- 🟡 **Updater has no cryptographic checksum.** `ui/updater.rs` verifies the
  downloaded installer's byte size against the release asset size, but does not
  verify a SHA-256. GitHub release assets expose no checksum field by default;
  a proper fix needs a published checksum asset plus a hashing dependency
  (e.g. `sha2`). Size + HTTPS via `curl` is the current mitigation.
- 🟡 **`panic = "abort"` in the release profile.** Any residual panic aborts
  without cleanup. Remaining `expect()` sites are limited to startup DB fallback
  (`app.rs`) and test code. Consider auditing before relying on release crash
  safety.
- 🟡 **`AppConfig.high_scores`** is serialized/deserialized but never read or
  written anywhere else — dead persisted state.
- 🟡 **Hardcoded 25/40 presets only** — no 60 s / 120-word options
  (`TimedDuration`, `WordCountTarget`).
- 🟡 **`SessionConfig.include_punctuation` / `include_numbers`** are never
  enabled by the UI; the generator paths exist but are unreachable in practice.
- 🟡 **`game/text.rs` `FAMOUS_QUOTES`** is now unused (kept as public data).
- 🟡 **Per-frame text layout** (`ui/typing_area.rs`) re-measures every glyph each
  frame. Fine for 25–40 word tests; a cached-layout approach would help very
  large custom passages.
- 🟡 **Accessibility** — icon-only window controls are painter-drawn (no
  screen-reader widget); no reduced-motion setting despite heavy animation.
- 🟡 **Minimum window 860×600** — the settings content clamps to `max(360.0)`;
  verify no clipping at the minimum size.

---

## Notes

- `db/models.rs` already defines a `PassageId { Passage(i64), LegacyText(i64) }`
  enum used by `update_passage`/`delete_passage`; the negative-id convention
  remains inside `get_all_passages_palette`/`search_passages` but is handled
  consistently.
- The repository contains two git roots: `typingforge/` (outer) and
  `typingforge/src/` (the crate). Build commands target the crate directory.
