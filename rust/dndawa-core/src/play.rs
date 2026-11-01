//! Terminal play: turning typed commands into actions and events into plain text.

use crate::attack::HitKind;
use crate::game::{Event, Game, GameAction, GameStatus};
use crate::grid::Direction;

/// One typed command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Act(GameAction),
    Save(String),
    Help,
    Quit,
}

/// Help text, kept short for a 97-column terminal.
pub const HELP: &str = "n s e w = move · a = attack · wait · save <file> · help · q = quit";

/// Read one line of input. Unknown input gives `None`.
pub fn parse_command(line: &str) -> Option<Command> {
    let line = line.trim();
    let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
    let action = |a| Some(Command::Act(a));
    match word.to_ascii_lowercase().as_str() {
        "n" | "north" => action(GameAction::Move(Direction::North)),
        "s" | "south" => action(GameAction::Move(Direction::South)),
        "e" | "east" => action(GameAction::Move(Direction::East)),
        "w" | "west" => action(GameAction::Move(Direction::West)),
        "a" | "attack" => action(GameAction::Attack),
        "wait" => action(GameAction::Wait),
        "save" if !rest.trim().is_empty() => Some(Command::Save(rest.trim().to_string())),
        "help" | "?" => Some(Command::Help),
        "q" | "quit" => Some(Command::Quit),
        _ => None,
    }
}

fn hit_word(kind: HitKind) -> &'static str {
    match kind {
        HitKind::Miss => "miss",
        HitKind::Hit => "hit",
        HitKind::Critical => "CRITICAL hit",
    }
}

/// One line describing what happened.
pub fn describe(event: &Event) -> String {
    match event {
        Event::Waited => "You wait.".to_string(),
        Event::Moved { to, foe: None } => {
            format!("You enter room {},{}. It is quiet.", to.x(), to.y())
        }
        Event::Moved {
            to,
            foe: Some(kind),
        } => {
            format!(
                "You enter room {},{}. A {kind:?} blocks the way!",
                to.x(),
                to.y()
            )
        }
        Event::Fought {
            foe,
            hero_attack,
            counter,
            foe_defeated,
        } => {
            let mut text = format!(
                "You roll {} vs {foe:?}: {} for {} damage.",
                hero_attack.natural,
                hit_word(hero_attack.kind),
                hero_attack.damage
            );
            if *foe_defeated {
                text.push_str(" It falls.");
            }
            if let Some(strike) = counter {
                text.push_str(&format!(
                    " It strikes back: {} for {} damage.",
                    hit_word(strike.kind),
                    strike.damage
                ));
            }
            text
        }
    }
}

/// Short status line: turn, hit points, position and the game's state.
pub fn status_line(game: &Game) -> String {
    let state = match game.status() {
        GameStatus::InProgress => "",
        GameStatus::Won => " · VICTORY: the Bone Warden is defeated",
        GameStatus::Lost => " · DEFEAT: the hero has fallen",
    };
    format!(
        "turn {} · HP {}/{} · room {},{}{state}",
        game.turn(),
        game.hero().hp(),
        game.hero().max_hp(),
        game.position().x(),
        game.position().y()
    )
}

#[cfg(test)]
mod tests {
    use super::{Command, describe, parse_command, status_line};
    use crate::game::{Event, Game, GameAction};
    use crate::grid::Direction;

    #[test]
    fn parses_commands_and_rejects_unknown_input() {
        assert_eq!(
            parse_command(" N "),
            Some(Command::Act(GameAction::Move(Direction::North)))
        );
        assert_eq!(parse_command("a"), Some(Command::Act(GameAction::Attack)));
        assert_eq!(
            parse_command("save run1.sav"),
            Some(Command::Save("run1.sav".into()))
        );
        assert_eq!(parse_command("save"), None);
        assert_eq!(parse_command("dance"), None);
        assert_eq!(parse_command("q"), Some(Command::Quit));
    }

    #[test]
    fn every_line_fits_a_97_column_terminal() {
        let mut game = Game::new(5);
        assert!(status_line(&game).len() <= 95);
        for action in [
            GameAction::Move(Direction::East),
            GameAction::Attack,
            GameAction::Wait,
        ] {
            if let Ok(event) = game.apply(action) {
                assert!(describe(&event).len() <= 95, "{}", describe(&event));
            }
        }
        assert!(describe(&Event::Waited).len() <= 95);
    }
}
