Here is the full directive for your agent. It’s structured to fix the immediate input bugs first, then lay down a robust, clean architecture for the database, and finally polish the WPM/live counter system. Copy-paste this directly to your agent.

---

## Agent Directive: Fix Input Logic, Establish Database Architecture, Polish Metrics

### Context
The typing app is functional but has a critical UX bug (caret jumps on typos) and lacks any persistence. We are moving to a robust architecture using SQLite with FTS5. The codebase must be modular, clean, and separated by concern. Do not lump everything into `main.rs`.

---

### PART 1 — Fix Typo Handling (PRIORITY: CRITICAL)

**Current Bug:** On a typo, the caret jumps forward, breaking flow.

**Required Behavior:**
- The caret **must NOT advance** on an incorrect keystroke.
- On a typo, **change the font color** of the mistyped character AND **apply a background highlight** to that word (or character) to signal the error visually.
- The user must **backspace** to correct the error before the caret continues.
- Do **not** use a jarring shake animation. Use color + background only. Subtle, elegant, decisive.

**Visual Spec:**
- Incorrect character foreground: soft crimson (e.g., `#FF5C5C`).
- Incorrect character background: low-opacity red (e.g., `#FF5C5C` at 15% alpha).
- Once the correct character is typed over the error, both foreground and background return to the default state (no lingering artifact).

**Logic Spec:**
- The "current index" in the typing state must be tied to *expected* characters, not total keystrokes.
- On `keypress`:
  - If `input_char == expected_char` → mark correct, advance index.
  - If `input_char != expected_char` → mark incorrect, **do not advance**, record the error in the keystroke log.
  - If `Backspace` → move index back, clear the error state on that character.

---

### PART 2 — Project Structure (NON-NEGOTIABLE)

Refactor the project into clean, separated modules. Each concern lives in its own folder/file. No god-files.

**Proposed layout:**
```
src/
├── main.rs                  // App entry, window setup only
├── app.rs                   // Root AppState, top-level orchestration
├── typing/
│   ├── mod.rs
│   ├── engine.rs            // Core typing logic, caret, index, correctness
│   ├── metrics.rs           // WPM, raw WPM, accuracy, consistency, burst
│   └── render.rs            // Rendering the typing canvas, caret, typo styling
├── db/
│   ├── mod.rs
│   ├── connection.rs        // SQLite pool/connection init, migrations trigger
│   ├── schema.rs            // CREATE TABLE statements, FTS5 setup
│   ├── models.rs            // Structs: User, Session, KeystrokeLog, Text, PB
│   ├── queries.rs           // Insert/select/update helpers
│   └── migrations.rs        // Versioned schema migrations
├── auth/
│   ├── mod.rs
│   └── local.rs             // Argon2 hashing, local-only user session
├── ui/
│   ├── mod.rs
│   ├── theme.rs             // Colors, fonts, spacing tokens
│   ├── panels/
│   │   ├── typing_view.rs
│   │   ├── results_view.rs
│   │   ├── settings.rs
│   │   └── editor.rs        // Tab-to-edit panel for custom text/paste
│   └── components.rs        // Reusable widgets: stat cards, heatmaps, etc.
├── data/
│   ├── mod.rs
│   ├── paths.rs             // OS-specific app data dir resolution
│   └── loader.rs            // Load/save JSON config, text packs
└── utils/
    ├── mod.rs
    └── text.rs              // Sanitization, normalization, char utils
```

**Rule:** Any new feature must land in the appropriate module. If a file exceeds ~400 lines, split it.

---

### PART 3 — Database (SQLite + FTS5)

**Location:** Store the SQLite file in the OS-specific app data directory (use the `directories` crate). Do **not** put the DB next to the binary.

**Schema (v1):**

- `users` — `id INTEGER PK`, `username TEXT UNIQUE`, `password_hash TEXT`, `created_at INTEGER`
- `sessions` — `id INTEGER PK`, `user_id INTEGER NULL` (NULL = guest), `mode TEXT`, `duration INTEGER`, `wpm REAL`, `raw_wpm REAL`, `accuracy REAL`, `consistency REAL`, `started_at INTEGER`, `ended_at INTEGER`
- `keystroke_logs` — `id INTEGER PK`, `session_id INTEGER`, `expected_char TEXT`, `actual_char TEXT`, `is_correct INTEGER`, `latency_ms INTEGER`, `position INTEGER`
- `texts` — `id INTEGER PK`, `title TEXT`, `content TEXT`, `char_count INTEGER`, `source TEXT` (seed | user_paste | user_edit), `created_by INTEGER NULL`, `created_at INTEGER`
- `texts_fts` — FTS5 virtual table over `title`, `content` (duplicated content for simplicity — do not use external-content triggers for v1)
- `personal_bests` — `user_id INTEGER`, `mode TEXT`, `duration INTEGER`, `wpm REAL`, `accuracy REAL`, `session_id INTEGER`, `achieved_at INTEGER`, PRIMARY KEY (`user_id`, `mode`, `duration`)

**Requirements:**
- Enable `PRAGMA journal_mode=WAL;` and `PRAGMA foreign_keys=ON;` on connection open.
- Versioned migrations: a `schema_version` table. On startup, run any pending migrations in order.
- All DB access goes through `db::queries`. Never write raw SQL in UI code.
- FTS5 queries must escape user input to avoid syntax errors.

**Guest handling:**
- Allow typing without login. `user_id = NULL` for guest sessions.
- Only persist sessions to DB if a user is logged in OR if the config explicitly enables guest history. Default: do **not** persist guest sessions to avoid DB bloat.

---

### PART 4 — WPM & Live Counter Polish

You already have WPM. Now make it feel *smooth*, not jittery.

**Rules:**
- Internal metrics update every keystroke.
- Rendered metrics update on a throttle: **every 150ms** or on a rolling 3-second window, whichever is cleaner.
- Apply **exponential moving average (EMA)** smoothing to the displayed WPM so it glides rather than snaps. Suggested alpha: `0.15`.
- Accuracy is cumulative (correct / total), not windowed.
- Consistency is the coefficient of variation of per-keystroke latencies — compute on session end, display live as a rough estimate.
- **Burst:** only show if it exceeds a threshold (e.g., > 100 WPM). Otherwise hide — flickering bursts are noise.

**Visual polish for counters:**
- Use tabular numerals (monospaced digits) so the numbers don’t shift width as they change.
- Dim the counters to 40% opacity while actively typing. Return to 100% when the test ends. Reduces visual noise and pulls focus to the text.

---

### PART 5 — Long-Form Passages (~200 chars)

- Default text blocks should be ~200 characters (one paragraph or a few sentences).
- Handle line wrapping gracefully in the typing canvas — no hyphenation, no mid-word breaks that confuse the caret.
- Ensure caret stays visible across wrapped lines. Auto-scroll if the block exceeds visible area.
- When a block is completed, transition to the next block with a soft fade, not a hard cut.

---

### PART 6 — Tab-to-Edit Panel

- Pressing **Tab** opens an editor panel where the user can paste or modify the current text block.
- Tab must NOT have two meanings. It opens the editor; **Esc** closes it. Do not bind Tab to focus-move while in this app.
- **Paste sanitization (mandatory):**
  - Strip zero-width chars (`\u200B`, `\u200C`, `\u200D`, `\uFEFF`).
  - Normalize smart quotes (`'`, `"`) to straight quotes.
  - Collapse multiple spaces into one.
  - Trim leading/trailing whitespace.
- Pasted/edited texts are saved to `texts` with `source='user_paste'` or `source='user_edit'` and become searchable via FTS5.
- Preserve caret position and scroll state when returning from the editor.

---

### PART 7 — Personal Best Detection

- After each test, compute score and compare against `personal_bests` for `(user_id, mode, duration)`.
- Tiebreakers in order: WPM → accuracy → consistency.
- If new PB: update the row, flag the session, and trigger the celebration banner.
- The celebration banner must be:
  - Brief (2–3 seconds auto-dismiss).
  - Skippable (any keystroke or click dismisses it).
  - Non-blocking (does not trap input or block "Play Again").

---

### PART 8 — Login (Local Only for Now)

- Users stored in SQLite with Argon2 password hashing (use the `argon2` crate).
- No network. No sync. Local desktop accounts only.
- Guest mode always available. Login is opt-in.
- Sessions table uses `user_id = NULL` for guests.

---

### Final Notes to the Agent

- **Robustness over cleverness.** Prefer explicit code over magic. No premature abstractions.
- **No god-files.** If you're tempted to add "just one more thing" to `main.rs`, don't.
- **Comment the non-obvious.** Especially around the caret index logic and FTS5 queries.
- **Test the caret bug fix first.** Do not proceed to Part 3 until Part 1 is verified working.
- **Do not touch the aesthetic theme tokens** (colors, fonts) beyond what Part 1 requires. The theme polish is a separate pass.

Ship Part 1 fully working before starting Part 2. We build on a stable foundation, not a moving one.
