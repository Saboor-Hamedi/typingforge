**PROMPT FOR AGENT:**

**Role:** Senior Rust GUI & Backend Engineer
**Project:** ForgeTyping / Velotype
**Task:** Two distinct workstreams. Complete them in order.

---

### PART 1: Fuzzy Search UI Overhaul (Critical Polish)
The current fuzzy search implementation is visually broken. Apply these specific fixes:

1.  **Input Field Sizing:** The input box is currently too large/tall. Reduce its height to a sleek, compact size (e.g., `h-12` or `48px`). It should feel like a command bar, not a text area.
2.  **Missing Icon:** There is no icon on the right side of the input. Add a subtle, muted magnifying glass icon (or `Ctrl+P` hint) aligned to the right edge inside the input container.
3.  **Placeholder Styling:** The placeholder text ("Search passages...") is ugly. Make it muted (low opacity gray), italicized slightly if it fits the theme, and ensure it is perfectly vertically centered.
4.  **Jumping Layout Bug:** When the user types, the fuzzy finder modal "jumps up" or shifts position.
    *   *Fix:* The modal must have a **fixed position** (centered on screen) and a **fixed maximum height** (e.g., `max-h-[60vh]`). The *internal list* should scroll, not the entire modal container.
5.  **Badge Background Removal:** The badges (Word count, Mode, Edit) currently have a "creamy layer" or solid background color behind the text.
    *   *Fix:* **Remove all background colors from badges.** They must be transparent. Use only text color (muted gray for info, accent color for "Edit") and perhaps a very thin, subtle border (1px, 10% opacity) if needed for definition.

---

### PART 2: Profile Data Backup & Import (New Feature Architecture)
Add robust data management to the `profile.rs` settings tab. This allows users to backup and restore their typing history.

**1. File Architecture:**
Do not clutter the main profile file. Create two dedicated modules in the same folder as `profile.rs` (e.g., `src/features/profile/` or `src/modules/profile/`):
*   **`typing_backup.rs`**: Handles all logic for exporting data.
*   **`typing_import.rs`**: Handles all logic for importing and validating data.

**2. UI Additions:**
*   In the Profile settings tab, next to the existing "Truncate" (or Delete) button, add two new buttons: **"Backup Data"** and **"Import Data"**.
*   Style them consistently with the existing buttons (e.g., outline style).

**3. Export Functionality (`typing_backup.rs`):**
*   When "Backup Data" is clicked, open a native **File Save Dialog** (using `rfd` or `native-dialog` crate).
*   Allow the user to choose the save location and filename (default: `velotype_history.json`).
*   Query the `typing_history` table (or whatever the main stats table is called) and serialize the data to a clean JSON format.
*   *JSON Structure Example (Strictly this format, NO version fields):*
    ```json
    {
      "exported_at": "2026-10-03T12:00:00Z",
      "records": [
        { "id": 1, "wpm": 60, "accuracy": 95.5, "date": "..." }
      ]
    }
    ```

**4. Import Functionality (`typing_import.rs`):**
*   When "Import Data" is clicked, open a native **File Open Dialog** filtering for `.json` files.
*   Read the file and parse the JSON using the logic defined in this module.
*   **Robust Error Handling (Crucial):**
    *   Validate the JSON structure. If it's missing the "records" array or has the wrong schema, show a sleek error toast: "Invalid file format."
    *   Handle column mismatches gracefully. If the JSON has extra fields, ignore them. If it's missing non-nullable fields, skip that record and log a warning (do not crash the app).
    *   Check for duplicate IDs. If a record already exists, either skip it or update it (your choice, but be consistent).
*   On success, insert the valid records into the `typing_history` table and show a success toast: "Imported X records successfully."

**5. Scope Restriction:**
*   This Backup/Import feature is **ONLY** for the typing history/stats table. Do not include settings, themes, or custom passages in this specific JSON file. Keep it focused.

---

**DELIVERABLES ORDER:**
1.  Fix all 5 Fuzzy Search UI issues.
2.  Create `typing_backup.rs` and implement Export JSON logic + File Dialog.
3.  Create `typing_import.rs` and implement Import JSON logic + Validation + File Dialog.
4.  Wire up the UI buttons in the Profile tab to these new modules.

**Note:** For the Import feature, use `serde_json` for parsing and ensure all database operations are wrapped in a `Result` type to prevent panics on bad data. Keep the JSON structure exactly as requested without any extra metadata fields.
