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
