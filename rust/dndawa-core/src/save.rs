//! Save files: a small versioned text format, one `key=value` per line.
//!
//! ```text
//! DNDAWA-SAVE 2
//! seed=5
//! turn=3
//! rng=1234567890
//! hero_hp=10
//! pos=1,0
//! rooms=-,rat:7,-,...   (25 entries, row by row; "-" is an empty room)
//! visited=1100000000...  (25 digits, row by row; 1 = the hero has been there)
//! ```
//!
//! Version 1 saves (no `visited` line) still load: the start room and the hero's room
//! count as visited.

use crate::GameState;
use crate::dice::Rng;
use crate::dungeon::{self, Dungeon, Foe};
use crate::game::{Game, new_hero};
use crate::grid::{Position, ROOMS};
use crate::monsters::MonsterKind;

/// First line of every save file written now. Bump the number when the format changes.
pub const HEADER: &str = "DNDAWA-SAVE 2";

/// First line of older saves, still accepted. They have no `visited` line.
pub const HEADER_V1: &str = "DNDAWA-SAVE 1";

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
    let visited: String = game
        .visited
        .iter()
        .map(|&seen| if seen { '1' } else { '0' })
        .collect();
    format!(
        "{HEADER}\nseed={}\nturn={}\nrng={}\nhero_hp={}\npos={},{}\nrooms={}\nvisited={visited}\n",
        game.seed(),
        game.turn(),
        game.rng.state(),
        game.hero().hp(),
        game.position().x(),
        game.position().y(),
        rooms.join(","),
    )
}

/// Why a save file was refused. Nothing is loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// First line is neither [`HEADER`] nor [`HEADER_V1`].
    BadHeader,
    /// A line is not `key=value`, or a key is unknown or repeated.
    BadLine(String),
    /// A required key is missing.
    Missing(&'static str),
    /// A value is malformed or out of range.
    BadValue(&'static str),
    /// Values are each valid but do not fit together (e.g. a boss outside its room).
    Inconsistent(&'static str),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadHeader => write!(f, "not a save file (expected {HEADER:?} or {HEADER_V1:?})"),
            Self::BadLine(line) => write!(f, "unexpected line: {line:?}"),
            Self::Missing(key) => write!(f, "missing {key}"),
            Self::BadValue(key) => write!(f, "invalid value for {key}"),
            Self::Inconsistent(what) => write!(f, "inconsistent save: {what}"),
        }
    }
}

impl std::error::Error for LoadError {}

const KEYS: [&str; 7] = ["seed", "turn", "rng", "hero_hp", "pos", "rooms", "visited"];

/// Read save-file text back into a game. Every field is checked.
pub fn from_save_str(text: &str) -> Result<Game, LoadError> {
    let mut lines = text.lines();
    let keys: &[&str] = match lines.next() {
        Some(HEADER) => &KEYS,
        Some(HEADER_V1) => &KEYS[..6],
        _ => return Err(LoadError::BadHeader),
    };
    let mut values: [Option<&str>; 7] = [None; 7];
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| LoadError::BadLine(line.to_string()))?;
        let slot = keys
            .iter()
            .position(|k| *k == key)
            .ok_or_else(|| LoadError::BadLine(line.to_string()))?;
        if values[slot].replace(value).is_some() {
            return Err(LoadError::BadLine(line.to_string()));
        }
    }
    let get = |i: usize| values[i].ok_or(LoadError::Missing(KEYS[i]));
    let number = |i: usize| -> Result<u64, LoadError> {
        get(i)?.parse().map_err(|_| LoadError::BadValue(KEYS[i]))
    };
    let seed = number(0)?;
    let turn = number(1)?;
    let rng_state = number(2)?;

    let mut hero = new_hero();
    let hero_hp = u32::try_from(number(3)?).map_err(|_| LoadError::BadValue("hero_hp"))?;
    if hero_hp > hero.max_hp() {
        return Err(LoadError::BadValue("hero_hp"));
    }
    hero.set_hp(hero_hp);

    let (x, y) = get(4)?.split_once(',').ok_or(LoadError::BadValue("pos"))?;
    let coord = |v: &str| v.parse::<u8>().map_err(|_| LoadError::BadValue("pos"));
    let position = Position::new(coord(x)?, coord(y)?).ok_or(LoadError::BadValue("pos"))?;

    let rooms = get(5)?
        .split(',')
        .map(parse_room)
        .collect::<Result<Vec<_>, _>>()?;
    if rooms.len() != ROOMS {
        return Err(LoadError::BadValue("rooms"));
    }
    check_layout(&rooms)?;

    let visited = if keys.len() == KEYS.len() {
        parse_visited(get(6)?)?
    } else {
        let mut seen = [false; ROOMS];
        seen[dungeon::index(dungeon::start())] = true;
        seen[dungeon::index(position)] = true;
        seen
    };
    check_visited(&visited, position, &rooms)?;
    let dungeon = Dungeon::from_rooms(rooms).ok_or(LoadError::BadValue("rooms"))?;

    Ok(Game {
        seed,
        state: GameState {
            turn,
            player_health: i32::try_from(hero.hp()).unwrap_or(i32::MAX),
        },
        hero,
        position,
        dungeon,
        rng: Rng::new(rng_state),
        visited,
    })
}

/// Exactly 25 digits, `0` or `1`.
fn parse_visited(value: &str) -> Result<[bool; ROOMS], LoadError> {
    let mut seen = [false; ROOMS];
    if value.len() != ROOMS {
        return Err(LoadError::BadValue("visited"));
    }
    for (slot, digit) in seen.iter_mut().zip(value.bytes()) {
        *slot = match digit {
            b'0' => false,
            b'1' => true,
            _ => return Err(LoadError::BadValue("visited")),
        };
    }
    Ok(seen)
}

/// The start room and the hero's room were entered, and the hero cannot leave a room
/// while a foe lives in it, so every other visited room must be empty.
fn check_visited(
    visited: &[bool; ROOMS],
    position: Position,
    rooms: &[Option<Foe>],
) -> Result<(), LoadError> {
    if !visited[dungeon::index(dungeon::start())] {
        return Err(LoadError::Inconsistent("start room not visited"));
    }
    if !visited[dungeon::index(position)] {
        return Err(LoadError::Inconsistent("hero's room not visited"));
    }
    let here = dungeon::index(position);
    let left_behind = (0..ROOMS).any(|i| i != here && visited[i] && rooms[i].is_some());
    if left_behind {
        return Err(LoadError::Inconsistent("a visited room still holds a foe"));
    }
    Ok(())
}

/// Rooms must fit the dungeon's fixed layout: the start room is empty and the boss
/// is either in its own room or already defeated.
fn check_layout(rooms: &[Option<Foe>]) -> Result<(), LoadError> {
    let (start, boss_room) = (
        dungeon::index(dungeon::start()),
        dungeon::index(dungeon::boss_room()),
    );
    if rooms[start].is_some() {
        return Err(LoadError::Inconsistent("start room must be empty"));
    }
    for (i, room) in rooms.iter().enumerate() {
        let is_boss = matches!(room, Some(foe) if foe.kind == MonsterKind::BoneWarden);
        if is_boss && i != boss_room {
            return Err(LoadError::Inconsistent("boss outside its room"));
        }
        if !is_boss && room.is_some() && i == boss_room {
            return Err(LoadError::Inconsistent("boss room holds another monster"));
        }
    }
    Ok(())
}

/// `-` for an empty room, or `code:hp` with 1 <= hp <= that monster's max HP.
fn parse_room(entry: &str) -> Result<Option<Foe>, LoadError> {
    if entry == "-" {
        return Ok(None);
    }
    let (code, hp) = entry.split_once(':').ok_or(LoadError::BadValue("rooms"))?;
    let kind = MonsterKind::from_code(code).ok_or(LoadError::BadValue("rooms"))?;
    let hp: u32 = hp.parse().map_err(|_| LoadError::BadValue("rooms"))?;
    let mut body = kind.spawn();
    if hp == 0 || hp > body.max_hp() {
        return Err(LoadError::BadValue("rooms"));
    }
    body.set_hp(hp);
    Ok(Some(Foe { kind, body }))
}

#[cfg(test)]
mod tests {
    use super::{HEADER, HEADER_V1, LoadError, from_save_str, to_save_string};
    use crate::dungeon;
    use crate::game::{Game, GameAction};
    use crate::grid::{Direction, Position};

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
            "visited=1000000000000000000000000",
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

    fn played(seed: u64) -> Game {
        let mut game = Game::new(seed);
        for action in [
            GameAction::Move(Direction::East),
            GameAction::Attack,
            GameAction::Attack,
            GameAction::Move(Direction::South),
            GameAction::Wait,
        ] {
            let _ = game.apply(action);
        }
        game
    }

    #[test]
    fn round_trip_gives_the_same_game_and_the_same_future() {
        for seed in 0..50 {
            let mut original = played(seed);
            let mut loaded = from_save_str(&to_save_string(&original)).unwrap();
            assert_eq!(loaded, original, "seed {seed}");
            for _ in 0..20 {
                assert_eq!(
                    loaded.apply(GameAction::Attack),
                    original.apply(GameAction::Attack)
                );
            }
            assert_eq!(loaded, original);
        }
    }

    #[test]
    fn damaged_saves_are_refused() {
        let good = to_save_string(&played(3));
        let bad = |from: &str, to: &str| from_save_str(&good.replacen(from, to, 1));
        assert_eq!(from_save_str("hello"), Err(LoadError::BadHeader));
        assert_eq!(bad(HEADER, "DNDAWA-SAVE 9"), Err(LoadError::BadHeader));
        assert_eq!(
            bad("seed=", "sneed="),
            Err(LoadError::BadLine("sneed=3".into()))
        );
        let moved: String = good
            .lines()
            .map(|l| if l.starts_with("pos=") { "pos=9,9" } else { l })
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(from_save_str(&moved), Err(LoadError::BadValue("pos")));
        assert_eq!(
            bad("hero_hp=", "hero_hp=99"),
            Err(LoadError::BadValue("hero_hp"))
        );
        assert_eq!(bad("boss:", "dragon:"), Err(LoadError::BadValue("rooms")));
        let without_rooms: String = good
            .lines()
            .filter(|l| !l.starts_with("rooms="))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(
            from_save_str(&without_rooms),
            Err(LoadError::Missing("rooms"))
        );
        let duplicated = format!("{good}turn=1\n");
        assert!(matches!(
            from_save_str(&duplicated),
            Err(LoadError::BadLine(_))
        ));
    }

    fn with_rooms(good: &str, rooms: &str) -> String {
        good.lines()
            .map(|l| if l.starts_with("rooms=") { rooms } else { l })
            .map(|l| format!("{l}\n"))
            .collect()
    }

    #[test]
    fn layout_that_cannot_happen_is_refused() {
        let good = to_save_string(&Game::new(1));
        let mut cells = vec!["-"; 25];
        cells[24] = "boss:45";
        let valid = format!("rooms={}", cells.join(","));
        assert!(from_save_str(&with_rooms(&good, &valid)).is_ok());

        cells[3] = "boss:45";
        let two_bosses = format!("rooms={}", cells.join(","));
        assert_eq!(
            from_save_str(&with_rooms(&good, &two_bosses)),
            Err(LoadError::Inconsistent("boss outside its room"))
        );

        let mut cells = vec!["-"; 25];
        cells[24] = "rat:7";
        let wrong_boss = format!("rooms={}", cells.join(","));
        assert_eq!(
            from_save_str(&with_rooms(&good, &wrong_boss)),
            Err(LoadError::Inconsistent("boss room holds another monster"))
        );

        let mut cells = vec!["-"; 25];
        cells[0] = "goblin:7";
        let busy_start = format!("rooms={}", cells.join(","));
        assert_eq!(
            from_save_str(&with_rooms(&good, &busy_start)),
            Err(LoadError::Inconsistent("start room must be empty"))
        );
    }

    fn with_line(good: &str, key: &str, line: Option<&str>) -> String {
        good.lines()
            .filter_map(|l| if l.starts_with(key) { line } else { Some(l) })
            .map(|l| format!("{l}\n"))
            .collect()
    }

    #[test]
    fn version_1_saves_still_load() {
        let game = played(3);
        let v2 = to_save_string(&game);
        let v1 = with_line(&v2, "visited=", None).replacen(HEADER, HEADER_V1, 1);
        let loaded = from_save_str(&v1).unwrap();
        assert_eq!(loaded.position(), game.position());
        for y in 0..5 {
            for x in 0..5 {
                let at = Position::new(x, y).unwrap();
                let expected = at == dungeon::start() || at == game.position();
                assert_eq!(loaded.visited(at), expected, "room {x},{y}");
            }
        }
        let v1_with_visited = v2.replacen(HEADER, HEADER_V1, 1);
        assert!(matches!(
            from_save_str(&v1_with_visited),
            Err(LoadError::BadLine(_))
        ));
    }

    #[test]
    fn damaged_visited_line_is_refused() {
        let good = to_save_string(&Game::new(1));
        let set = |v: &str| from_save_str(&with_line(&good, "visited=", Some(v)));
        assert_eq!(
            from_save_str(&with_line(&good, "visited=", None)),
            Err(LoadError::Missing("visited"))
        );
        assert_eq!(set("visited=1"), Err(LoadError::BadValue("visited")));
        let mut digits = vec!["0"; 25];
        digits[0] = "1";
        digits[7] = "x";
        let line = format!("visited={}", digits.concat());
        assert_eq!(set(&line), Err(LoadError::BadValue("visited")));
        assert_eq!(
            set(&format!("visited={}", "0".repeat(25))),
            Err(LoadError::Inconsistent("start room not visited"))
        );
        assert_eq!(
            set(&format!("visited={}", "1".repeat(25))),
            Err(LoadError::Inconsistent("a visited room still holds a foe"))
        );
    }

    #[test]
    fn hero_must_stand_in_a_visited_room() {
        let good = to_save_string(&Game::new(1));
        let moved = with_line(&good, "pos=", Some("pos=1,0"));
        let mut cells = vec!["-"; 25];
        cells[24] = "boss:45";
        let quiet = with_line(
            &moved,
            "rooms=",
            Some(&format!("rooms={}", cells.join(","))),
        );
        assert_eq!(
            from_save_str(&quiet),
            Err(LoadError::Inconsistent("hero's room not visited"))
        );
    }

    #[test]
    fn defeated_boss_still_loads() {
        let good = to_save_string(&Game::new(1));
        let cleared = format!("rooms={}", vec!["-"; 25].join(","));
        assert!(from_save_str(&with_rooms(&good, &cleared)).is_ok());
    }
}
