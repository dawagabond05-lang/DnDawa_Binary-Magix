//! `dndawa-crawl`: play the dungeon in a terminal.
//!
//! Usage: `dndawa-crawl [--seed N] [--load FILE]`

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use dndawa_core::game::{Game, GameStatus};
use dndawa_core::play::{self, Command};
use dndawa_core::save;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
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
    show(&game, &journal);
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
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
        show(&game, &journal);
        let _ = io::stdout().flush();
        if game.status() != GameStatus::InProgress {
            break;
        }
    }
    ExitCode::SUCCESS
}

/// Print the screen, then the journal (last lines, newest last), then the status line.
fn show(game: &Game, journal: &play::Journal) {
    for line in play::screen(game) {
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
        _ => Err("usage: dndawa-crawl [--seed N] [--load FILE]".to_string()),
    }
}
