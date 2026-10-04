# TypingForge ("Velotype") — Forward Backlog & Completion Status

All items below were implemented. Each is marked ✅ with the concrete change made.
Verified with `cargo check --all-targets` and `cargo test --lib` (16/16 passing).

---

## 1. Security / Robustness

- ✅ **Updater SHA-256 verification.** `ui/updater.rs` now looks for a published
  checksum manifest (`checksums.txt` / `sha256sums.txt` / `<asset>.sha256`), downloads
  it, parses the `sha256sum`-style entry for the installer, and verifies the file hash
  before `ReadyToInstall`. Falls back to the existing byte-size check (with a logged
  warning) when no manifest is published. Added the `sha2` dependency.
- ✅ **Release `panic = "abort"` removed.** Release profile now unwinds, so a panic in
  a worker thread no longer aborts the whole process.
- ✅ **Password zeroization.** `zeroize` added; `profile.rs` now wipes the password and
  confirm-password buffers (`String::zeroize`) on successful auth/registration.
- ✅ **Auth input hardening.** `auth/local.rs` validates username by character count
  (2–32), rejects embedded whitespace, and measures the password by characters.
- ✅ **Import/export error surfacing.** Export returns `"Export cancelled by user."` on
  cancel; import distinguishes `"Import cancelled by user."` from parse errors.

## 2. Features / Config Gaps

- ✅ **60s / 120-word presets.** `TimedDuration::Sec60` and `WordCountTarget::Words120`
  added; header mode switcher and custom-texts UI expose them; timed generation scales
  with duration; personal-best keys and passage selection handle the new values.
- ✅ **Punctuation / numbers wired.** `AppConfig.include_punctuation` /
  `include_numbers` added (persisted under gameplay settings), two toggles added to the
  HUD & Motion tab, and `app.rs` propagates them into `SessionConfig` each frame.
- ✅ **`high_scores` dead field removed** from `AppConfig` and `SettingsFile`
  (personal bests are authoritative in SQLite).
- ✅ **`FAMOUS_QUOTES` removed** along with the unused quote generator.
- ✅ **Guest PB sentinel named.** `GUEST_USER_ID` constant added in `db/models.rs` and
  used by `app.rs` instead of the magic `0`.

## 3. Performance

- ✅ **Typing-view layout cache.** `ui/typing_area.rs` caches line wrapping, glyph
  metrics and per-word origins in egui memory, keyed by word-length signature + width +
  font size. The per-frame `layout_no_wrap("M")` and wrapping pass are skipped on cache
  hits.
- ✅ **Velocity-graph cache.** `ui/velocity_graph.rs` caches the resampled instant
  polyline and Catmull-Rom spline, keyed by data signature and canvas geometry.
- ✅ **Event-driven profile stats.** A `sessions_version` counter is threaded through
  `VelotypeApp` → `SettingsPanel` → `ProfileTab`; the aggregate query only re-runs when
  the user or session history changes.
- ✅ **Idle repaint verified.** Repaint is already gated by
  `needs_continuous_repaint` with a 30 ms idle tick only on the typing screen.

## 4. UX / Polish

- ✅ **Reduced-motion setting.** `AppConfig.reduced_motion` + toggle in HUD & Motion;
  disables particles and screen shake and makes the caret snap instantly
  (`CaretController.reduced_motion`).
- ✅ **Accessibility.** Window min/max/close buttons and mode tabs now expose
  `WidgetInfo` labels and hover tooltips for screen readers.
- ✅ **Min-size layout.** Header mode-switcher width tuned so it does not collide with
  the right-side update/profile controls at the 860 px minimum width.
- ✅ **Footer tooltips.** Hovering the footer telemetry shows an explanation of
  Net / Raw / Accuracy / Streak.
- ✅ **Esc semantics.** Esc on Results now leaves to Typing (matching the on-screen
  "Back to Typing" action) instead of restarting; the Results footer hint reflects this.
- ✅ **Naming reconciled.** User-facing strings standardized on **Velotype**;
  repository/package/data-path identifiers documented in `README.md`.

## 5. Repo Hygiene

- ✅ **`publish` subcommand removed** from `main.rs`; the app binary is purely the app.
  Release automation remains available via `cargo run --bin publish`.
- ✅ **`.gitignore` verified** (`/target` and build artifacts ignored).
- ✅ **Layout documented** in `README.md` (crate lives in `src/`; binary list; naming
  table).

---

## Notes / not changed

- The outer `typingforge/` git root is a stale gitlink to the `src/` crate repo; git
  internals were intentionally left untouched to avoid corrupting history. Layout is
  documented in `README.md`.
- `include_punctuation` / `include_numbers` affect generated drill text only (DB-loaded
  passages are unaffected), by design.
