//! `dndawa-crawl`: play the dungeon in a terminal.
//!
//! Usage: `dndawa-crawl [--seed N | --load FILE] [--color]`

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use dndawa_core::game::{Game, GameStatus};
use dndawa_core::play::{self, Command};
use dndawa_core::save;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Colour only on request. The game never colours by default, so NO_COLOR has nothing
    // to turn off, and an explicit --color wins (no-color.org: per-command flags override it).
    let (colour, args) = take_colour_flag(&args);
    let mut game = match start(&args) {
        Ok(game) => game,
        Err(message) => {
            eprintln!("dndawa-crawl: {message}");
            return ExitCode::from(2);
        }
    };
    println!("DnDawa_Binary-Magix · seed {}", game.seed());
    println!("{}", play::HELP);
    let mut journal = play::Journal::default();
    show(&game, &journal, colour);
    let stdin = io::stdin();
    let mut input_failed = false;
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            // A line that is not UTF-8 is skipped; the game goes on.
            Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                journal.push("Unreadable input (not UTF-8): line skipped.");
                show(&game, &journal, colour);
                continue;
            }
            Err(error) => {
                eprintln!("dndawa-crawl: cannot read input: {error}");
                input_failed = true;
                break;
            }
        };
        let message = match play::parse_command(&line) {
            Some(Command::Act(action)) => match game.apply(action) {
                Ok(event) => play::describe(&event),
                Err(error) => format!("Refused: {error}."),
            },
            Some(Command::Save(name)) => match play::save_path(&name) {
                Err(why) => format!("Not saved: {why}."),
                Ok(path) => {
                    let written = std::fs::create_dir_all(play::SAVE_DIR)
                        .and_then(|()| std::fs::write(&path, save::to_save_string(&game)));
                    match written {
                        Ok(()) => format!("Saved to {}.", path.display()),
                        Err(error) => format!("Could not save: {error}."),
                    }
                }
            },
            Some(Command::Help) => play::HELP.to_string(),
            Some(Command::Quit) => break,
            None => format!("Unknown command. {}", play::HELP),
        };
        journal.push(message);
        show(&game, &journal, colour);
        let _ = io::stdout().flush();
        if game.status() != GameStatus::InProgress {
            break;
        }
    }
    // The game ended, or the player quit with `q` or by closing the input (Ctrl-D).
    for line in play::summary(&game) {
        println!("{line}");
    }
    if input_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Take out `--color` flags, reading the arguments in order: the value that follows
/// `--seed` or `--load` is never a flag, so a save file may even be named `--color`.
fn take_colour_flag(args: &[String]) -> (bool, Vec<String>) {
    let mut colour = false;
    let mut rest = Vec::with_capacity(args.len());
    let mut items = args.iter();
    while let Some(arg) = items.next() {
        if arg == "--seed" || arg == "--load" {
            rest.push(arg.clone());
            rest.extend(items.next().cloned());
        } else if arg == "--color" {
            colour = true;
        } else {
            rest.push(arg.clone());
        }
    }
    (colour, rest)
}

/// Print the screen, then the journal (last lines, newest last), then the status line.
fn show(game: &Game, journal: &play::Journal, colour: bool) {
    let lines = if colour {
        play::screen_coloured(game)
    } else {
        play::screen(game)
    };
    for line in lines {
        println!("{line}");
    }
    for line in journal.lines() {
        println!("{line}");
    }
    println!("{}", play::status_line(game));
}

/// Build the starting game from `--seed N` or `--load FILE` (default seed 1).
fn start(args: &[String]) -> Result<Game, String> {
    match args {
        [] => Ok(Game::new(1)),
        [flag, seed] if flag == "--seed" => seed
            .parse()
            .map(Game::new)
            .map_err(|_| format!("--seed needs a whole number, got {seed:?}")),
        [flag, path] if flag == "--load" => {
            let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            save::from_save_str(&text).map_err(|e| format!("{path}: {e}"))
        }
        _ => Err("usage: dndawa-crawl [--seed N | --load FILE] [--color]".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::take_colour_flag;

    fn split(args: &[&str]) -> (bool, Vec<String>) {
        let owned: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        take_colour_flag(&owned)
    }

    #[test]
    fn color_is_a_flag_anywhere_but_never_the_value_of_load_or_seed() {
        assert_eq!(split(&[]), (false, vec![]));
        assert_eq!(split(&["--color"]), (true, vec![]));
        assert_eq!(
            split(&["--color", "--seed", "5"]),
            (true, vec!["--seed".into(), "5".into()])
        );
        assert_eq!(
            split(&["--seed", "5", "--color"]),
            (true, vec!["--seed".into(), "5".into()])
        );
        assert_eq!(
            split(&["--load", "--color"]),
            (false, vec!["--load".into(), "--color".into()])
        );
        assert_eq!(
            split(&["--load", "--color", "--color"]),
            (true, vec!["--load".into(), "--color".into()])
        );
        assert_eq!(split(&["--load"]), (false, vec!["--load".into()]));
    }
}
