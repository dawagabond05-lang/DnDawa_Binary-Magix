//! Terminal play: turning typed commands into actions and events into plain text.

use crate::attack::HitKind;
use crate::board::render_board;
use crate::card::card;
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
pub const HELP: &str = "n s e w = move · a = attack · wait · save <name> · help · q = quit";

/// Folder that holds save files (git-ignored).
pub const SAVE_DIR: &str = "saves";

/// Turn a save name into `saves/<name>.sav`. Only a plain name is accepted: no path
/// separators, no leading dot, letters, digits, `-` and `_` only, at most 40 characters.
pub fn save_path(name: &str) -> Result<std::path::PathBuf, &'static str> {
    let stem = name.strip_suffix(".sav").unwrap_or(name);
    let plain = !stem.is_empty()
        && stem.len() <= 40
        && !stem.starts_with('.')
        && stem
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if plain {
        Ok(std::path::Path::new(SAVE_DIR).join(format!("{stem}.sav")))
    } else {
        Err("use a plain save name: letters, digits, - or _ (max 40)")
    }
}

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

/// Lines kept in the journal shown under the screen.
pub const JOURNAL_LINES: usize = 3;

/// The last few things that happened, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Journal {
    lines: std::collections::VecDeque<String>,
}

impl Journal {
    /// Add a line, forgetting the oldest beyond [`JOURNAL_LINES`].
    pub fn push(&mut self, line: impl Into<String>) {
        self.lines.push_back(line.into());
        while self.lines.len() > JOURNAL_LINES {
            self.lines.pop_front();
        }
    }

    /// Lines to print: older ones marked `·`, the newest marked `»`; each at most 95
    /// characters.
    pub fn lines(&self) -> Vec<String> {
        let last = self.lines.len().saturating_sub(1);
        self.lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let mark = if i == last { "»" } else { "·" };
                format!("{mark} {line}").chars().take(95).collect()
            })
            .collect()
    }
}

/// Key to the board tokens.
pub const LEGEND: &str = "@ you · ? fog · · empty · B boss · r s g foes";

/// The whole screen: the board on the left, a `◄` pointing at the hero's row, and
/// the cards on the right (the hero, then the foe in the room as the target).
/// At most 11 lines, each at most 95 characters.
pub fn screen(game: &Game) -> Vec<String> {
    let mut side = vec![String::new()];
    side.extend(card("Hero", game.hero()));
    if let Some(foe) = game.dungeon().foe(game.position()) {
        side.push(String::new());
        side.extend(card(&format!("Target: {}", foe.body.name), &foe.body));
    }
    let board = render_board(game);
    side.resize(board.len() - 1, String::new());
    side.push(LEGEND.to_string());
    let hero_row = 1 + 2 * usize::from(game.position().y());
    board
        .into_iter()
        .zip(side)
        .enumerate()
        .map(|(i, (left, right))| {
            let gutter = if i == hero_row { " ◄ " } else { "   " };
            format!("{left}{gutter}{right}").trim_end().to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Command, Journal, describe, parse_command, save_path, screen, status_line};
    use crate::dungeon;
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
    fn save_names_stay_inside_the_saves_folder() {
        assert_eq!(
            save_path("run1").unwrap(),
            std::path::Path::new("saves/run1.sav")
        );
        assert_eq!(
            save_path("run1.sav").unwrap(),
            std::path::Path::new("saves/run1.sav")
        );
        for bad in [
            "",
            "../x",
            "/etc/passwd",
            "a/b",
            ".hidden",
            "sp ace",
            &"x".repeat(41),
        ] {
            assert!(save_path(bad).is_err(), "{bad:?} should be refused");
        }
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

    #[test]
    fn journal_keeps_the_last_three_lines_newest_last() {
        let mut journal = Journal::default();
        assert!(journal.lines().is_empty());
        journal.push("one");
        assert_eq!(journal.lines(), ["» one"]);
        for line in ["two", "three", "four"] {
            journal.push(line);
        }
        assert_eq!(journal.lines(), ["· two", "· three", "» four"]);
        journal.push("x".repeat(200));
        assert_eq!(journal.lines()[2].chars().count(), 95);
    }

    #[test]
    fn screen_fits_and_points_at_the_hero() {
        let game = Game::new(5);
        let lines = screen(&game);
        assert_eq!(lines.len(), 11);
        for line in &lines {
            assert!(line.chars().count() <= 95, "{line}");
        }
        assert!(lines[1].contains(" ◄ ┌─ Hero"), "{}", lines[1]);
        assert_eq!(lines.iter().filter(|l| l.contains('◄')).count(), 1);
        assert!(!lines.iter().any(|l| l.contains("Target:")));
        assert!(lines[10].ends_with(super::LEGEND));
    }

    #[test]
    fn screen_shows_the_foe_card_as_the_target() {
        let mut game = Game::new(5);
        game.position = dungeon::boss_room();
        let lines = screen(&game);
        assert_eq!(lines.len(), 11);
        assert!(lines.iter().any(|l| l.contains("Target: Bone Warden")));
        assert!(lines[9].contains("│@ B│ ◄ └"), "{}", lines[9]);
        for line in &lines {
            assert!(line.chars().count() <= 95, "{line}");
        }
    }
}
