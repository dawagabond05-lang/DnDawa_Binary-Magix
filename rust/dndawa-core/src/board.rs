//! The board: the 5x5 rooms drawn as a grid, with fog over rooms not yet visited.
//!
//! ```text
//! ┌───┬───┬───┬───┬───┐
//! │ · │@ g│ ? │ ? │ ? │
//! ├───┼───┼───┼───┼───┤
//! ...
//! │ ? │ ? │ ? │ ? │ B │
//! └───┴───┴───┴───┴───┘
//! ```
//!
//! Tokens: `@` hero · `?` not visited yet · `·` visited and empty · `B` the boss room
//! (always shown while the boss lives) · `r` `s` `g` a foe (rat, skeleton, goblin).

use crate::dungeon;
use crate::game::Game;
use crate::grid::{Position, SIZE};
use crate::monsters::MonsterKind;

/// Lines of text drawing the board (11 lines, 21 columns).
pub fn render_board(game: &Game) -> Vec<String> {
    let edge = |left: &str, join: &str, right: &str| {
        let cells = vec!["───"; usize::from(SIZE)];
        format!("{left}{}{right}", cells.join(join))
    };
    let mut lines = vec![edge("┌", "┬", "┐")];
    for y in 0..SIZE {
        let cells: Vec<String> = (0..SIZE)
            .map(|x| cell(game, Position::new(x, y).expect("inside grid")))
            .collect();
        lines.push(format!("│{}│", cells.join("│")));
        if y + 1 < SIZE {
            lines.push(edge("├", "┼", "┤"));
        }
    }
    lines.push(edge("└", "┴", "┘"));
    lines
}

/// One-letter token for a foe.
pub fn token(kind: MonsterKind) -> char {
    match kind {
        MonsterKind::GiantRat => 'r',
        MonsterKind::Skeleton => 's',
        MonsterKind::Goblin => 'g',
        MonsterKind::BoneWarden => 'B',
    }
}

/// Three characters for one room.
fn cell(game: &Game, at: Position) -> String {
    let foe = game.dungeon().foe(at).map(|foe| token(foe.kind));
    if at == game.position() {
        match foe {
            Some(t) => format!("@ {t}"),
            None => " @ ".to_string(),
        }
    } else if at == dungeon::boss_room() && foe.is_some() {
        " B ".to_string()
    } else if !game.visited(at) {
        " ? ".to_string()
    } else {
        match foe {
            Some(t) => format!(" {t} "),
            None => " · ".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render_board;
    use crate::dungeon;
    use crate::game::{Game, GameAction};
    use crate::grid::Direction;

    fn row(lines: &[String], y: usize) -> Vec<String> {
        lines[1 + 2 * y]
            .trim_matches('│')
            .split('│')
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn new_board_is_fog_except_start_and_boss() {
        let lines = render_board(&Game::new(5));
        assert_eq!(lines.len(), 11);
        for line in &lines {
            assert_eq!(line.chars().count(), 21, "{line}");
        }
        assert_eq!(lines[0], "┌───┬───┬───┬───┬───┐");
        assert_eq!(lines[10], "└───┴───┴───┴───┴───┘");
        assert_eq!(row(&lines, 0), [" @ ", " ? ", " ? ", " ? ", " ? "]);
        assert_eq!(row(&lines, 4), [" ? ", " ? ", " ? ", " ? ", " B "]);
    }

    #[test]
    fn visited_rooms_lift_the_fog_and_foes_show_beside_the_hero() {
        let seed = (0..500)
            .find(|&s| {
                let east = dungeon::start().step(Direction::East).unwrap();
                Game::new(s).dungeon().foe(east).is_some()
            })
            .expect("some seed puts a monster next to the start");
        let mut game = Game::new(seed);
        game.apply(GameAction::Move(Direction::East)).unwrap();
        let cells = row(&render_board(&game), 0);
        assert_eq!(cells[0], " · ");
        assert!(cells[1].starts_with("@ "), "{cells:?}");
        assert!(matches!(cells[1].chars().last(), Some('r' | 's' | 'g')));
    }

    #[test]
    fn hero_in_the_boss_room_stands_beside_the_boss() {
        let mut game = Game::new(5);
        game.position = dungeon::boss_room();
        assert_eq!(row(&render_board(&game), 4)[4], "@ B");
    }
}
