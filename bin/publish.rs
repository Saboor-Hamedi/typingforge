use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

// ─── Colours (ANSI) ──────────────────────────────────────────────────────────
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

fn step(msg: &str) {
    println!("\n{BOLD}{CYAN}▶  {msg}{RESET}");
}

fn ok(msg: &str) {
    println!("{GREEN}✓  {msg}{RESET}");
}

#[allow(dead_code)]
fn warn(msg: &str) {
    println!("{YELLOW}⚠  {msg}{RESET}");
}

fn bail(msg: &str) -> ! {
    eprintln!("{RED}{BOLD}✗  {msg}{RESET}");
    std::process::exit(1);
}

// ─── Shell helpers ────────────────────────────────────────────────────────────
fn run(program: &str, args: &[&str], cwd: &Path) {
    run_with_retry(program, args, cwd, 1);
}

fn run_with_retry(program: &str, args: &[&str], cwd: &Path, retries: usize) {
    for attempt in 1..=retries {
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .status();

        match status {
            Ok(s) if s.success() => return,
            Ok(s) if attempt < retries => {
                warn(&format!(
                    "`{program} {}` exited with status {s}. Retrying in 2s (attempt {attempt}/{retries})...",
                    args.join(" ")
                ));
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Ok(s) => bail(&format!("`{program} {}` exited with status {s}", args.join(" "))),
            Err(e) if attempt < retries => {
                warn(&format!(
                    "Failed to execute {program}: {e}. Retrying in 2s (attempt {attempt}/{retries})..."
                ));
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => bail(&format!("failed to execute {program}: {e}")),
        }
    }
}

#[allow(dead_code)]
fn run_output(program: &str, args: &[&str], cwd: &Path) -> String {
    let out = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| bail(&format!("failed to run {program}: {e}")));

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail(&format!("`{program} {}` failed: {stderr}", args.join(" ")));
    }

    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

// ─── Version helpers ──────────────────────────────────────────────────────────
fn read_version(cargo_toml: &Path) -> (String, String) {
    let content = fs::read_to_string(cargo_toml)
        .unwrap_or_else(|e| bail(&format!("Cannot read {:?}: {e}", cargo_toml)));

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                let ver = val.trim().trim_matches('"').to_string();
                return (ver, content);
            }
        }
    }
    bail("Could not find `version` field in Cargo.toml");
}

fn bump_patch(version: &str) -> String {
    let parts: Vec<u64> = version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();

    if parts.len() != 3 {
        bail(&format!("Unexpected version format: {version}"));
    }
    format!("{}.{}.{}", parts[0], parts[1], parts[2] + 1)
}

fn write_version(cargo_toml: &Path, content: &str, old: &str, new: &str) {
    // Replace only the first occurrence of the package version line
    let new_content = content.replacen(
        &format!("version = \"{old}\""),
        &format!("version = \"{new}\""),
        1,
    );
    fs::write(cargo_toml, new_content)
        .unwrap_or_else(|e| bail(&format!("Cannot write {:?}: {e}", cargo_toml)));
}

fn update_inno_setup_version(inno_path: &Path, new: &str) {
    if !inno_path.exists() {
        return;
    }
    if let Ok(content) = fs::read_to_string(inno_path) {
        let mut new_lines = Vec::new();
        for line in content.lines() {
            if line.starts_with("#define MyAppVersion ") {
                new_lines.push(format!("#define MyAppVersion \"{new}\""));
            } else {
                new_lines.push(line.to_string());
            }
        }
        let _ = fs::write(inno_path, new_lines.join("\r\n") + "\r\n");
    }
}

// ─── Git helpers ──────────────────────────────────────────────────────────────
fn git_is_clean(repo: &Path) -> bool {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .unwrap_or_else(|e| bail(&format!("git status failed: {e}")));
    out.stdout.is_empty()
}

fn tag_exists(repo: &Path, tag: &str) -> bool {
    let out = Command::new("git")
        .args(["tag", "-l", tag])
        .current_dir(repo)
        .output()
        .unwrap_or_else(|e| bail(&format!("git tag failed: {e}")));
    !String::from_utf8_lossy(&out.stdout).trim().is_empty()
}

fn ensure_clean_remote(cwd: &Path) {
    let clean_url = "https://github.com/Saboor-Hamedi/typingforge.git";
    let check = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output();

    if let Ok(out) = check {
        if out.status.success() {
            let _ = Command::new("git")
                .args(["remote", "set-url", "origin", clean_url])
                .current_dir(cwd)
                .output();
        } else {
            let _ = Command::new("git")
                .args(["remote", "add", "origin", clean_url])
                .current_dir(cwd)
                .output();
        }
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────
fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Resolve project root (whether run from root or src/ or any subfolder)
    let mut cwd = std::env::current_dir().expect("cannot get cwd");
    if cwd.join("src").join("Cargo.toml").exists() {
        cwd = cwd.join("src");
    } else {
        while !cwd.join("Cargo.toml").exists() {
            if let Some(parent) = cwd.parent() {
                cwd = parent.to_path_buf();
            } else {
                bail(&format!(
                    "Cargo.toml not found in any parent directories. start_cwd={:?}",
                    std::env::current_dir().ok()
                ));
            }
        }
    }

    let cargo_toml = cwd.join("Cargo.toml");

    // Auto-initialize Git if .git folder is missing
    if !cwd.join(".git").exists() {
        step("Initializing Git repository");
        run("git", &["init"], &cwd);
        let _ = Command::new("git").args(["branch", "-M", "master"]).current_dir(&cwd).output();
        ok("Initialized git repository (master branch)");
    }

    ensure_clean_remote(&cwd);

    println!("\n{BOLD}╔══════════════════════════════════════════╗");
    println!("║     typingforge Publish Automation       ║");
    println!("╚══════════════════════════════════════════╝{RESET}\n");

    // ── 1. Read current version ───────────────────────────────────────────────
    step("Reading current version from Cargo.toml");
    let (current_ver, content) = read_version(&cargo_toml);
    ok(&format!("Current version: v{current_ver}"));

    // ── 2. Determine new version ──────────────────────────────────────────────
    let new_ver = if args.len() > 1 && args[1].starts_with(|c: char| c.is_ascii_digit()) {
        args[1].clone()
    } else {
        let bumped = bump_patch(&current_ver);
        print!("\n{YELLOW}New version [{bumped}]? (press Enter to accept, or type a version): {RESET}");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            bumped
        } else {
            trimmed.to_string()
        }
    };
    ok(&format!("Publishing version: v{new_ver}"));

    // ── 3. Check working tree ─────────────────────────────────────────────────
    step("Checking working tree");
    if !git_is_clean(&cwd) {
        let _ = Command::new("git")
            .args(["status", "--short"])
            .current_dir(&cwd)
            .status();
        print!("\n{YELLOW}Working tree has uncommitted changes. Stage & include all files in v{new_ver}? [Y/n]: {RESET}");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim().to_lowercase();
        if trimmed == "n" || trimmed == "no" {
            bail("Aborted: Commit or stash your changes before publishing.");
        }
    }
    ok("Working tree ready");

    // ── 4. Check tag doesn't already exist ────────────────────────────────────
    let tag = format!("v{new_ver}");
    if tag_exists(&cwd, &tag) {
        print!("{YELLOW}Tag {tag} already exists locally. Delete and recreate? [y/N]: {RESET}");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim().to_lowercase();
        if trimmed == "y" || trimmed == "yes" {
            let _ = Command::new("git")
                .args(["tag", "-d", &tag])
                .current_dir(&cwd)
                .output();
            ok(&format!("Removed old local tag {tag}"));
        } else {
            bail(&format!("Tag {tag} already exists. Bump to a higher version."));
        }
    }

    // ── 5. Run tests ──────────────────────────────────────────────────────────
    step("Running tests (cargo test)");
    run("cargo", &["test"], &cwd);
    ok("All tests passed");

    // ── 6. Write bumped version ───────────────────────────────────────────────
    step(&format!("Bumping version: {current_ver} → {new_ver}"));
    write_version(&cargo_toml, &content, &current_ver, &new_ver);
    ok(&format!("Wrote v{new_ver} to Cargo.toml"));

    let inno_path = if cwd.join("installer").join("inno_setup.iss").exists() {
        cwd.join("installer").join("inno_setup.iss")
    } else {
        cwd.join("src").join("installer").join("inno_setup.iss")
    };
    update_inno_setup_version(&inno_path, &new_ver);
    ok(&format!("Updated installer/inno_setup.iss to v{new_ver}"));

    // Also update Cargo.lock by touching it via cargo check
    step("Updating Cargo.lock (cargo check)");
    run("cargo", &["check", "--quiet"], &cwd);
    ok("Cargo.lock updated");

    // ── 7. Git commit ─────────────────────────────────────────────────────────
    step("Committing version bump & release files");
    run("git", &["add", "-A"], &cwd);
    run(
        "git",
        &["commit", "-m", &format!("chore: release v{new_ver}")],
        &cwd,
    );
    ok(&format!("Committed: chore: release v{new_ver}"));

    // ── 8. Git tag ────────────────────────────────────────────────────────────
    step(&format!("Tagging release: {tag}"));
    run(
        "git",
        &["tag", "-a", &tag, "-m", &format!("Release {tag}")],
        &cwd,
    );
    ok(&format!("Created annotated tag {tag}"));

    // ── 9. Push ───────────────────────────────────────────────────────────────
    step("Pushing commits and tag to GitHub");
    ensure_clean_remote(&cwd);
    run_with_retry("git", &["push", "-u", "origin", "master"], &cwd, 3);
    run_with_retry("git", &["push", "origin", &tag], &cwd, 3);
    run_with_retry("git", &["push", "--tags"], &cwd, 3);
    ok("Pushed to GitHub — CI/CD pipeline is now running!");

    println!("\n{BOLD}{GREEN}════════════════════════════════════════");
    println!("  ✓  Release v{new_ver} published!");
    println!("  GitHub Actions will now:");
    println!("  • Build on Windows, macOS, Linux");
    println!("  • Create a GitHub Release with binaries");
    println!("  • Upload installers (.exe, portable .zip, .tar.gz)");
    println!("════════════════════════════════════════{RESET}\n");

    println!("  View your release at:");
    println!("  {CYAN}https://github.com/Saboor-Hamedi/typingforge/releases/tag/{tag}{RESET}\n");
}
