//! Save files: a small versioned text format, one `key=value` per line.
//!
//! ```text
//! DNDAWA-SAVE 1
//! seed=5
//! turn=3
//! rng=1234567890
//! hero_hp=10
//! pos=1,0
//! rooms=-,rat:7,-,...   (25 entries, row by row; "-" is an empty room)
//! ```

use crate::game::Game;

/// First line of every save file. Bump the number when the format changes.
pub const HEADER: &str = "DNDAWA-SAVE 1";

/// Write the game as save-file text. Loading it back gives the same game.
pub fn to_save_string(game: &Game) -> String {
    let rooms: Vec<String> = game
        .dungeon()
        .rooms()
        .iter()
        .map(|room| match room {
            None => "-".to_string(),
            Some(foe) => format!("{}:{}", foe.kind.code(), foe.body.hp()),
        })
        .collect();
    format!(
        "{HEADER}\nseed={}\nturn={}\nrng={}\nhero_hp={}\npos={},{}\nrooms={}\n",
        game.seed(),
        game.turn(),
        game.rng.state(),
        game.hero().hp(),
        game.position().x(),
        game.position().y(),
        rooms.join(","),
    )
}

#[cfg(test)]
mod tests {
    use super::{HEADER, to_save_string};
    use crate::game::{Game, GameAction};
    use crate::grid::Direction;

    #[test]
    fn save_has_header_and_every_field() {
        let text = to_save_string(&Game::new(5));
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], HEADER);
        for key in [
            "seed=5",
            "turn=0",
            "rng=",
            "hero_hp=12",
            "pos=0,0",
            "rooms=",
        ] {
            assert!(lines.iter().any(|l| l.starts_with(key)), "missing {key}");
        }
        let rooms = lines.iter().find(|l| l.starts_with("rooms=")).unwrap();
        assert_eq!(rooms["rooms=".len()..].split(',').count(), 25);
        assert!(rooms.ends_with("boss:45"));
    }

    #[test]
    fn save_reflects_play() {
        let mut game = Game::new(5);
        game.apply(GameAction::Wait).unwrap();
        let _ = game.apply(GameAction::Move(Direction::South));
        let text = to_save_string(&game);
        assert!(text.contains(&format!("turn={}", game.turn())));
        assert!(text.contains(&format!(
            "pos={},{}",
            game.position().x(),
            game.position().y()
        )));
    }
}
