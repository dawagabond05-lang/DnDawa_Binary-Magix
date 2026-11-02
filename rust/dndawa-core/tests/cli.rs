//! Runs the real `dndawa-crawl` binary: how it ends, and what it does with odd input.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run the binary with `args`, feed it `input`, return (stdout, stderr, exit code).
fn run(args: &[&str], input: &[u8], env: &[(&str, &str)]) -> (String, String, i32) {
    run_in(&std::env::temp_dir(), args, input, env)
}

/// Same as [`run`], with the working directory set to `dir`.
fn run_in(
    dir: &std::path::Path,
    args: &[&str],
    input: &[u8],
    env: &[(&str, &str)],
) -> (String, String, i32) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_dndawa-crawl"));
    command
        .current_dir(dir)
        .args(args)
        .env_remove("NO_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in env {
        command.env(key, value);
    }
    let mut child = command.spawn().expect("the binary starts");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(input)
        .expect("input is written");
    let out = child.wait_with_output().expect("the binary finishes");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

const LEFT: &str = "The game is left in progress.";

#[test]
fn q_ends_with_the_summary() {
    let (out, _, code) = run(&["--seed", "2"], b"q\n", &[]);
    assert_eq!(code, 0);
    assert!(out.contains(LEFT), "{out}");
}

#[test]
fn end_of_input_also_ends_with_the_summary() {
    let (out, err, code) = run(&["--seed", "2"], b"e\n", &[]);
    assert_eq!(code, 0, "{err}");
    let last: Vec<&str> = out.lines().rev().take(2).collect();
    assert_eq!(last[1], LEFT, "{out}");
    assert!(
        last[0].starts_with("turns 1 · rooms explored 2/25"),
        "{out}"
    );
}

#[test]
fn a_line_that_is_not_utf8_is_skipped_and_play_goes_on() {
    let (out, _, code) = run(&["--seed", "2"], b"\xff\xfe\ne\nq\n", &[]);
    assert_eq!(code, 0);
    assert!(
        out.contains("Unreadable input (not UTF-8): line skipped."),
        "{out}"
    );
    assert!(out.contains("turns 1 · rooms explored 2/25"), "{out}");
}

#[test]
fn a_bad_flag_prints_usage_and_exits_2() {
    let (_, err, code) = run(&["--bogus"], b"", &[]);
    assert_eq!(code, 2);
    assert!(err.contains("usage: dndawa-crawl"), "{err}");
}

fn has_colour(text: &str) -> bool {
    text.contains('\x1b')
}

#[test]
fn no_colour_unless_asked_whatever_no_color_says() {
    assert!(!has_colour(&run(&["--seed", "2"], b"q\n", &[]).0));
    assert!(!has_colour(
        &run(&["--seed", "2"], b"q\n", &[("NO_COLOR", "1")]).0
    ));
}

#[test]
fn an_explicit_color_flag_wins_over_no_color() {
    assert!(has_colour(&run(&["--color", "--seed", "2"], b"q\n", &[]).0));
    let (out, _, code) = run(&["--seed", "2", "--color"], b"q\n", &[("NO_COLOR", "1")]);
    assert_eq!(code, 0);
    assert!(has_colour(&out));
}

#[test]
fn a_save_file_named_color_still_loads() {
    let dir = std::env::temp_dir().join(format!("dndawa-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (out, _, code) = run_in(&dir, &["--seed", "7"], b"save probe\nq\n", &[]);
    assert_eq!(code, 0, "{out}");
    std::fs::rename(dir.join("saves/probe.sav"), dir.join("--color")).unwrap();
    let (plain, err, code) = run_in(&dir, &["--load", "--color"], b"q\n", &[]);
    assert_eq!(code, 0, "{err}");
    assert!(plain.contains("seed 7"), "{plain}");
    assert!(!has_colour(&plain));
    let (painted, _, code) = run_in(&dir, &["--load", "--color", "--color"], b"q\n", &[]);
    assert_eq!(code, 0);
    assert!(painted.contains("seed 7") && has_colour(&painted));
    std::fs::remove_dir_all(&dir).unwrap();
}

// Edge cases first written by Young.Sage_02 (handoff YS02-HANDOFF-cli-edge-tests, patch
// lost with its cloud container), rebuilt here with the Council's IMPROVE notes: a fresh
// directory per test (LUMEN), a real save for the --seed/--load conflict (UMBRA CLI), and
// whole-line comparison of replies (AvvA_22).

/// An empty directory of its own for one test, removed again when the test ends, even if it
/// fails (UMBRA CLI).
struct TempDir(std::path::PathBuf);

impl TempDir {
    /// A fresh directory owned by this test: the name carries the pid, the clock and a
    /// counter, and `create_dir` refuses a path that already exists, so no older directory
    /// is reused or deleted. A name that is already taken is retried with the next counter.
    fn new(name: &str) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        for _ in 0..100 {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let unique = format!("dndawa-cli-{}-{nanos}-{n}-{name}", std::process::id());
            let dir = std::env::temp_dir().join(unique);
            match std::fs::create_dir(&dir) {
                Ok(()) => return Self(dir),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("could not create {}: {e}", dir.display()),
            }
        }
        panic!("no free test directory name after 100 tries");
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    /// Removes the directory. A failed removal fails the test, unless the test is already
    /// panicking (a second panic would abort and hide the first failure).
    fn drop(&mut self) {
        match std::fs::remove_dir_all(&self.0) {
            Err(e) if !std::thread::panicking() => {
                panic!("could not remove {}: {e}", self.0.display())
            }
            _ => {}
        }
    }
}

#[test]
fn a_failing_test_still_removes_its_directory() {
    let mut kept = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let dir = TempDir::new("unwind");
        kept = Some(dir.path().to_path_buf());
        panic!("simulated test failure");
    }));
    assert!(result.is_err());
    let path = kept.expect("the directory was created");
    assert!(!path.exists(), "left behind: {}", path.display());
}

#[cfg(unix)]
#[test]
fn a_failed_removal_fails_the_test() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new("stuck");
    let locked = dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::write(locked.join("file"), "x").unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();
    // A privileged user ignores the read-only bit: probe it instead of guessing the uid.
    let privileged = match std::fs::write(locked.join("probe"), "x") {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(e) => panic!("probe write failed for another reason: {e}"),
    };
    let path = dir.path().to_path_buf();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || drop(dir)));
    if path.exists() {
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::remove_dir_all(&path).unwrap();
    }
    if !privileged {
        assert!(
            result.is_err(),
            "Drop must fail the test when removal fails"
        );
    }
}

#[test]
fn two_directories_with_the_same_name_never_collide() {
    let a = TempDir::new("same");
    let b = TempDir::new("same");
    assert_ne!(a.path(), b.path());
    assert!(a.path().is_dir() && b.path().is_dir());
}

const USAGE: &str = "dndawa-crawl: usage: dndawa-crawl [--seed N | --load FILE] [--color]";

/// Runs in `dir`, expects exit 2, no game on stdout, and returns stderr.
fn refused(dir: &std::path::Path, args: &[&str]) -> String {
    let (out, err, code) = run_in(dir, args, b"", &[]);
    assert_eq!(code, 2, "{args:?}: {err}");
    assert!(out.is_empty(), "{args:?} must not start a game: {out}");
    err
}

#[test]
fn a_missing_value_or_an_extra_argument_shows_the_usage() {
    let dir = TempDir::new("usage");
    for args in [&["--seed"][..], &["--load"], &["--seed", "1", "extra"]] {
        assert_eq!(refused(dir.path(), args).trim_end(), USAGE, "{args:?}");
    }
}

#[test]
fn a_bad_seed_gets_a_message_and_no_usage() {
    let dir = TempDir::new("seed");
    for bad in ["abc", "-1", "99999999999999999999999"] {
        let err = refused(dir.path(), &["--seed", bad]);
        assert_eq!(
            err.trim_end(),
            format!("dndawa-crawl: --seed needs a whole number, got {bad:?}")
        );
    }
}

#[test]
fn a_missing_or_damaged_save_file_names_the_file() {
    let dir = TempDir::new("load");
    let err = refused(dir.path(), &["--load", "no-such.sav"]);
    assert!(err.starts_with("dndawa-crawl: no-such.sav: "), "{err}");
    assert!(!err.contains("usage:"), "{err}");
    std::fs::write(dir.path().join("bad.sav"), "hello\n").unwrap();
    let err = refused(dir.path(), &["--load", "bad.sav"]);
    assert!(
        err.starts_with("dndawa-crawl: bad.sav: not a save file"),
        "{err}"
    );
}

#[test]
fn seed_and_load_together_show_the_usage_even_with_a_real_save() {
    let dir = TempDir::new("conflict");
    let (_, _, code) = run_in(dir.path(), &["--seed", "3"], b"save probe\nq\n", &[]);
    assert_eq!(code, 0);
    assert!(dir.path().join("saves/probe.sav").is_file());
    let err = refused(dir.path(), &["--seed", "1", "--load", "saves/probe.sav"]);
    assert_eq!(err.trim_end(), USAGE);
}

#[test]
fn replies_are_shown_whole() {
    use dndawa_core::play::HELP;
    for (input, expected) in [
        ("dance\nq\n", format!("» Unknown command. {HELP}")),
        ("help\nq\n", format!("» {HELP}")),
    ] {
        let (out, _, code) = run(&["--seed", "2"], input.as_bytes(), &[]);
        assert_eq!(code, 0);
        assert!(
            out.lines().any(|l| l == expected),
            "{expected:?} not in:\n{out}"
        );
    }
}

#[test]
fn every_printed_line_fits_95_columns() {
    // No reply is longer than 95 today, so this checks the whole output of a varied session;
    // the cut of a longer line itself is unit-tested in play::Journal (DESTINY's note).
    let input = format!("help\ndance\nsave {}\ne\na\nwait\nq\n", "x".repeat(41));
    let (out, _, code) = run(&["--seed", "2"], input.as_bytes(), &[]);
    assert_eq!(code, 0);
    for line in out.lines() {
        let width = line.chars().count();
        assert!(width <= 95, "{width} columns: {line}");
    }
}
