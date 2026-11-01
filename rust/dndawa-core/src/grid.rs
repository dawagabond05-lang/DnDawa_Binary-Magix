//! The 5x5 grid of rooms: positions and moves between them.

/// Rooms per side.
pub const SIZE: u8 = 5;

/// Rooms in the whole grid.
pub const ROOMS: usize = SIZE as usize * SIZE as usize;

/// A room coordinate, always inside the grid (`x` east, `y` south, both `0..SIZE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    x: u8,
    y: u8,
}

/// A step to a neighbouring room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Why a move was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// The step would leave the grid.
    Wall,
}

impl std::fmt::Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wall => f.write_str("a wall blocks the way"),
        }
    }
}

impl std::error::Error for MoveError {}

impl Position {
    /// Make a position, or `None` if it would be outside the grid.
    pub fn new(x: u8, y: u8) -> Option<Self> {
        (x < SIZE && y < SIZE).then_some(Self { x, y })
    }

    pub fn x(self) -> u8 {
        self.x
    }

    pub fn y(self) -> u8 {
        self.y
    }

    /// The neighbouring position, or [`MoveError::Wall`] at the edge.
    pub fn step(self, direction: Direction) -> Result<Self, MoveError> {
        let (x, y) = match direction {
            Direction::North => (Some(self.x), self.y.checked_sub(1)),
            Direction::South => (Some(self.x), self.y.checked_add(1)),
            Direction::East => (self.x.checked_add(1), Some(self.y)),
            Direction::West => (self.x.checked_sub(1), Some(self.y)),
        };
        match (x, y) {
            (Some(x), Some(y)) => Self::new(x, y).ok_or(MoveError::Wall),
            _ => Err(MoveError::Wall),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, MoveError, Position, SIZE};

    #[test]
    fn positions_outside_the_grid_are_rejected() {
        assert!(Position::new(0, 0).is_some());
        assert!(Position::new(SIZE - 1, SIZE - 1).is_some());
        assert!(Position::new(SIZE, 0).is_none());
        assert!(Position::new(0, SIZE).is_none());
    }

    #[test]
    fn corners_have_walls_on_two_sides() {
        let top_left = Position::new(0, 0).unwrap();
        assert_eq!(top_left.step(Direction::North), Err(MoveError::Wall));
        assert_eq!(top_left.step(Direction::West), Err(MoveError::Wall));
        let bottom_right = Position::new(SIZE - 1, SIZE - 1).unwrap();
        assert_eq!(bottom_right.step(Direction::South), Err(MoveError::Wall));
        assert_eq!(bottom_right.step(Direction::East), Err(MoveError::Wall));
    }

    #[test]
    fn steps_move_one_room() {
        let centre = Position::new(2, 2).unwrap();
        assert_eq!(
            centre.step(Direction::North),
            Ok(Position::new(2, 1).unwrap())
        );
        assert_eq!(
            centre.step(Direction::South),
            Ok(Position::new(2, 3).unwrap())
        );
        assert_eq!(
            centre.step(Direction::East),
            Ok(Position::new(3, 2).unwrap())
        );
        assert_eq!(
            centre.step(Direction::West),
            Ok(Position::new(1, 2).unwrap())
        );
    }
}
