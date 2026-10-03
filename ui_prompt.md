cargo run --bin publish
   Compiling forgetyping v0.1.10 (B:\rust\velotype-desktop-rust\src)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.97s
     Running `target\debug\publish.exe`

╔══════════════════════════════════════════╗
║     typingforge Publish Automation       ║
╚══════════════════════════════════════════╝


▶  Reading current version from Cargo.toml
✓  Current version: v0.1.10

New version [0.1.11]? (press Enter to accept, or type a version):
✓  Publishing version: v0.1.11

▶  Checking working tree
✓  Working tree ready

▶  Running tests (cargo test)
   Compiling forgetyping v0.1.10 (B:\rust\velotype-desktop-rust\src)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.58s
     Running unittests lib.rs (target\debug\deps\forgetyping-5e2456118dfae8a1.exe)

running 9 tests
test game::stats::tests::test_accuracy_calculation ... ok
test game::stats::tests::test_wpm_calculation ... ok
test typing::metrics::tests::test_ema_smoothing_and_burst_threshold ... ok
test utils::text::tests::test_sanitize_smart_quotes_and_zero_width ... ok
test utils::text::tests::test_escape_fts5_query ... ok
test typing::engine::tests::test_typos_advance_caret_and_backspace_recovers ... ok
test typing::engine::tests::test_completes_immediately_on_last_letter_without_space ... ok
test db::queries::tests::test_in_memory_db_migrations_and_queries ... ok
test auth::local::tests::test_argon2_hash_and_verify ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.92s

     Running unittests main.rs (target\debug\deps\forgetyping-cddab03f958afeea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests bin\publish.rs (target\debug\deps\publish-1af84d0dd4b7370a.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests forgetyping

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

✓  All tests passed

▶  Bumping version: 0.1.10 → 0.1.11
✓  Wrote v0.1.11 to Cargo.toml
✓  Updated installer/inno_setup.iss to v0.1.11

▶  Updating Cargo.lock (cargo check)
✓  Cargo.lock updated

▶  Committing version bump & release files
warning: in the working copy of 'Cargo.lock', LF will be replaced by CRLF the next time Git touches it
warning: in the working copy of 'Cargo.toml', LF will be replaced by CRLF the next time Git touches it
[master 195ce62] chore: release v0.1.11
 3 files changed, 3 insertions(+), 3 deletions(-)
✓  Committed: chore: release v0.1.11

▶  Tagging release: v0.1.11
✓  Created annotated tag v0.1.11

▶  Pushing commits and tag to GitHub
fatal: unable to access 'https://github.com/Saboor-Hamedi/typingforge.git/': Failed to connect to github.com port 443 after 21095 ms: Could not connect to server
⚠  `git push -u origin master` exited with status exit code: 128. Retrying in 2s (attempt 1/3)...
remote: Repository not found.
fatal: repository 'https://github.com/Saboor-Hamedi/typingforge.git/' not found
⚠  `git push -u origin master` exited with status exit code: 128. Retrying in 2s (attempt 2/3)...
remote: Repository not found.
fatal: repository 'https://github.com/Saboor-Hamedi/typingforge.git/' not found
✗  `git push -u origin master` exited with status exit code: 128
error: process didn't exit successfully: `target\debug\publish.exe` (exit code: 1)
@Saboor ➜ src git(master) 
