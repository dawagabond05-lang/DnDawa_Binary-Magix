//! A whole game: the hero, where they stand, the dungeon and the turn counter.
//! Rule: a refused action changes nothing, not even the turn counter.

use crate::combatant::Combatant;
use crate::dice::Rng;
use crate::dungeon::{self, Dungeon};
use crate::grid::{Direction, MoveError, Position};
use crate::monsters::MonsterKind;
use crate::{Action, ActionError, GameState, apply_action};

/// What the player asks for on their turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    Wait,
    Move(Direction),
}

/// What happened on a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Waited,
    /// The hero moved; `foe` is what waits in the new room, if anything.
    Moved {
        to: Position,
        foe: Option<MonsterKind>,
    },
}

/// Why an action was refused. The game is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameError {
    Turn(ActionError),
    Move(MoveError),
    /// A living foe in this room blocks the way out.
    FoeBlocks,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Turn(error) => error.fmt(f),
            Self::Move(error) => error.fmt(f),
            Self::FoeBlocks => f.write_str("a foe blocks the way out"),
        }
    }
}

impl std::error::Error for GameError {}

/// One game in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub(crate) seed: u64,
    pub(crate) state: GameState,
    pub(crate) hero: Combatant,
    pub(crate) position: Position,
    pub(crate) dungeon: Dungeon,
    pub(crate) rng: Rng,
}

/// The hero's statblock (HOMEBREW, level-1 fighter style).
pub fn new_hero() -> Combatant {
    Combatant::new("Hero", 12, 16, 5, "1d8+3".parse().expect("valid dice"))
}

impl Game {
    /// Start a new game. The same seed always gives the same dungeon and rolls.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let dungeon = Dungeon::generate(&mut rng);
        let hero = new_hero();
        let state = GameState {
            turn: 0,
            player_health: i32::try_from(hero.hp()).unwrap_or(i32::MAX),
        };
        Self {
            seed,
            state,
            hero,
            position: dungeon::start(),
            dungeon,
            rng,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn turn(&self) -> u64 {
        self.state.turn
    }

    pub fn hero(&self) -> &Combatant {
        &self.hero
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn dungeon(&self) -> &Dungeon {
        &self.dungeon
    }

    /// Take one action. On `Err` the game is exactly as it was.
    pub fn apply(&mut self, action: GameAction) -> Result<Event, GameError> {
        match action {
            GameAction::Wait => {
                self.advance_turn()?;
                Ok(Event::Waited)
            }
            GameAction::Move(direction) => {
                if self.dungeon.foe(self.position).is_some() {
                    return Err(GameError::FoeBlocks);
                }
                let to = self.position.step(direction).map_err(GameError::Move)?;
                self.advance_turn()?;
                self.position = to;
                Ok(Event::Moved {
                    to,
                    foe: self.dungeon.foe(to).map(|foe| foe.kind),
                })
            }
        }
    }

    /// Count the turn. Fails (changing nothing) at `u64::MAX`.
    fn advance_turn(&mut self) -> Result<(), GameError> {
        apply_action(&mut self.state, Action::Wait).map_err(GameError::Turn)
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, Game, GameAction, GameError};
    use crate::ActionError;
    use crate::dungeon;
    use crate::grid::{Direction, MoveError};

    #[test]
    fn new_game_starts_at_turn_zero_in_the_start_room() {
        let game = Game::new(5);
        assert_eq!(game.turn(), 0);
        assert_eq!(game.position(), dungeon::start());
        assert_eq!(game.hero().hp(), 12);
    }

    #[test]
    fn wait_and_moves_count_turns() {
        let mut game = Game::new(5);
        assert_eq!(game.apply(GameAction::Wait), Ok(Event::Waited));
        let event = game.apply(GameAction::Move(Direction::East)).unwrap();
        assert!(matches!(event, Event::Moved { .. }));
        assert_eq!(game.turn(), 2);
    }

    #[test]
    fn walking_into_a_wall_changes_nothing() {
        let mut game = Game::new(5);
        let before = game.clone();
        assert_eq!(
            game.apply(GameAction::Move(Direction::North)),
            Err(GameError::Move(MoveError::Wall))
        );
        assert_eq!(game, before);
    }

    #[test]
    fn a_living_foe_blocks_leaving_its_room() {
        // Find a seed whose first room east holds a monster.
        let seed = (0..500)
            .find(|&s| {
                let game = Game::new(s);
                let east = dungeon::start().step(Direction::East).unwrap();
                game.dungeon().foe(east).is_some()
            })
            .expect("some seed puts a monster next to the start");
        let mut game = Game::new(seed);
        game.apply(GameAction::Move(Direction::East)).unwrap();
        let before = game.clone();
        assert_eq!(
            game.apply(GameAction::Move(Direction::West)),
            Err(GameError::FoeBlocks)
        );
        assert_eq!(game, before);
    }

    #[test]
    fn turn_overflow_changes_nothing() {
        let mut game = Game::new(5);
        game.state.turn = u64::MAX;
        let before = game.clone();
        assert_eq!(
            game.apply(GameAction::Move(Direction::East)),
            Err(GameError::Turn(ActionError::TurnOverflow))
        );
        assert_eq!(game, before);
    }
}
