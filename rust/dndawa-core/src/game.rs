//! A whole game: the hero, where they stand, the dungeon and the turn counter.
//! Rule: a refused action changes nothing, not even the turn counter.

use crate::attack::{AttackOutcome, attack};
use crate::combatant::Combatant;
use crate::dice::Rng;
use crate::dungeon::{self, Dungeon};
use crate::grid::{Direction, MoveError, Position, ROOMS};
use crate::monsters::MonsterKind;
use crate::{Action, ActionError, GameState, apply_action};

/// What the player asks for on their turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    Wait,
    Move(Direction),
    /// Attack the foe in the hero's room.
    Attack,
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
    /// The hero attacked; `counter` is the foe's reply if it survived.
    Fought {
        foe: MonsterKind,
        hero_attack: AttackOutcome,
        counter: Option<AttackOutcome>,
        foe_defeated: bool,
    },
}

/// Where the game stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    /// The boss is defeated.
    Won,
    /// The hero has fallen.
    Lost,
}

/// Why an action was refused. The game is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameError {
    Turn(ActionError),
    Move(MoveError),
    /// A living foe in this room blocks the way out.
    FoeBlocks,
    /// There is nothing to attack in this room.
    NothingToAttack,
    /// The game has ended; no more actions are taken.
    GameOver,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Turn(error) => error.fmt(f),
            Self::Move(error) => error.fmt(f),
            Self::FoeBlocks => f.write_str("a foe blocks the way out"),
            Self::NothingToAttack => f.write_str("there is nothing to attack here"),
            Self::GameOver => f.write_str("the game is over"),
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
    /// Rooms the hero has entered, row by row (the start room counts).
    pub(crate) visited: [bool; ROOMS],
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
        let mut visited = [false; ROOMS];
        visited[dungeon::index(dungeon::start())] = true;
        Self {
            seed,
            state,
            hero,
            position: dungeon::start(),
            dungeon,
            rng,
            visited,
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

    /// Has the hero ever entered this room?
    pub fn visited(&self, at: Position) -> bool {
        self.visited[dungeon::index(at)]
    }

    /// Won once the boss room is empty, lost once the hero has 0 HP.
    pub fn status(&self) -> GameStatus {
        if !self.hero.is_alive() {
            GameStatus::Lost
        } else if self.dungeon.foe(dungeon::boss_room()).is_none() {
            GameStatus::Won
        } else {
            GameStatus::InProgress
        }
    }

    /// Take one action. On `Err` the game is exactly as it was.
    pub fn apply(&mut self, action: GameAction) -> Result<Event, GameError> {
        if self.status() != GameStatus::InProgress {
            return Err(GameError::GameOver);
        }
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
                self.visited[dungeon::index(to)] = true;
                Ok(Event::Moved {
                    to,
                    foe: self.dungeon.foe(to).map(|foe| foe.kind),
                })
            }
            GameAction::Attack => self.attack(),
        }
    }

    /// Hero strikes the foe in the room; a surviving foe strikes back.
    fn attack(&mut self) -> Result<Event, GameError> {
        let target_ac = match self.dungeon.foe(self.position) {
            Some(foe) => foe.body.armor_class,
            None => return Err(GameError::NothingToAttack),
        };
        // Count the turn before rolling, so a refused turn consumes no dice.
        self.advance_turn()?;
        let hero_attack = attack(&self.hero, target_ac, &mut self.rng);
        let foe = self.dungeon.foe_mut(self.position).expect("checked above");
        let kind = foe.kind;
        foe.body.take_damage(hero_attack.damage);
        let foe_defeated = !foe.body.is_alive();
        let counter = if foe_defeated {
            self.dungeon.clear(self.position);
            None
        } else {
            let strike = attack(&foe.body, self.hero.armor_class, &mut self.rng);
            self.hero.take_damage(strike.damage);
            self.state.player_health = i32::try_from(self.hero.hp()).unwrap_or(i32::MAX);
            Some(strike)
        };
        Ok(Event::Fought {
            foe: kind,
            hero_attack,
            counter,
            foe_defeated,
        })
    }

    /// Count the turn. Fails (changing nothing) at `u64::MAX`.
    fn advance_turn(&mut self) -> Result<(), GameError> {
        apply_action(&mut self.state, Action::Wait).map_err(GameError::Turn)
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, Game, GameAction, GameError, GameStatus};
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
    fn rooms_are_marked_visited_when_entered() {
        let mut game = Game::new(5);
        let east = dungeon::start().step(Direction::East).unwrap();
        assert!(game.visited(dungeon::start()));
        assert!(!game.visited(east));
        assert!(!game.visited(dungeon::boss_room()));
        game.apply(GameAction::Move(Direction::East)).unwrap();
        assert!(game.visited(east));
        let before = game.clone();
        let _ = game.apply(GameAction::Move(Direction::North));
        assert_eq!(game, before, "a refused move marks nothing");
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
    fn attacking_an_empty_room_changes_nothing() {
        let mut game = Game::new(5);
        let before = game.clone();
        assert_eq!(
            game.apply(GameAction::Attack),
            Err(GameError::NothingToAttack)
        );
        assert_eq!(game, before);
    }

    #[test]
    fn fighting_ends_with_the_foe_cleared_or_the_hero_hurt() {
        let mut game = Game::new(5);
        game.position = dungeon::boss_room();
        let mut fights = 0;
        while game.dungeon().foe(game.position()).is_some() && game.hero().is_alive() {
            match game.apply(GameAction::Attack).unwrap() {
                Event::Fought {
                    foe_defeated,
                    counter,
                    ..
                } => {
                    assert_eq!(foe_defeated, counter.is_none());
                }
                other => panic!("unexpected {other:?}"),
            }
            fights += 1;
            assert!(fights < 1000);
        }
        let hp = u32::try_from(game.state.player_health).unwrap();
        assert_eq!(hp, game.hero().hp());
    }

    #[test]
    fn defeating_the_boss_wins_and_ends_the_game() {
        let mut game = Game::new(5);
        assert_eq!(game.status(), GameStatus::InProgress);
        game.dungeon.clear(dungeon::boss_room());
        assert_eq!(game.status(), GameStatus::Won);
        let before = game.clone();
        assert_eq!(game.apply(GameAction::Wait), Err(GameError::GameOver));
        assert_eq!(game, before);
    }

    #[test]
    fn a_fallen_hero_loses() {
        let mut game = Game::new(5);
        game.hero.take_damage(u32::MAX);
        assert_eq!(game.status(), GameStatus::Lost);
        assert_eq!(game.apply(GameAction::Wait), Err(GameError::GameOver));
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
