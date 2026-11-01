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
    println!("{}", play::status_line(&game));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        match play::parse_command(&line) {
            Some(Command::Act(action)) => match game.apply(action) {
                Ok(event) => println!("{}", play::describe(&event)),
                Err(error) => println!("Refused: {error}."),
            },
            Some(Command::Save(name)) => match play::save_path(&name) {
                Err(why) => println!("Not saved: {why}."),
                Ok(path) => {
                    let written = std::fs::create_dir_all(play::SAVE_DIR)
                        .and_then(|()| std::fs::write(&path, save::to_save_string(&game)));
                    match written {
                        Ok(()) => println!("Saved to {}.", path.display()),
                        Err(error) => println!("Could not save: {error}."),
                    }
                }
            },
            Some(Command::Help) => println!("{}", play::HELP),
            Some(Command::Quit) => break,
            None => println!("Unknown command. {}", play::HELP),
        }
        println!("{}", play::status_line(&game));
        let _ = io::stdout().flush();
        if game.status() != GameStatus::InProgress {
            break;
        }
    }
    ExitCode::SUCCESS
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
