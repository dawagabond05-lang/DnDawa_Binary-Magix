//! The dungeon: what waits in each of the 5x5 rooms. Built from a seed, so the same
//! seed always builds the same dungeon.

use crate::combatant::Combatant;
use crate::dice::Rng;
use crate::grid::{Position, SIZE};
use crate::monsters::MonsterKind;

/// A monster standing in a room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foe {
    pub kind: MonsterKind,
    pub body: Combatant,
}

/// The 5x5 rooms. The hero starts in the north-west corner; the boss waits in the
/// far south-east corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dungeon {
    rooms: Vec<Option<Foe>>,
}

/// Where the hero starts.
pub fn start() -> Position {
    Position::new(0, 0).expect("inside grid")
}

/// Where the boss waits.
pub fn boss_room() -> Position {
    Position::new(SIZE - 1, SIZE - 1).expect("inside grid")
}

/// Row-by-row index of a room, `0..ROOMS`.
pub(crate) fn index(at: Position) -> usize {
    usize::from(at.y()) * usize::from(SIZE) + usize::from(at.x())
}

impl Dungeon {
    /// Build a dungeon. Each ordinary room holds a monster with chance 4 in 10.
    pub fn generate(rng: &mut Rng) -> Self {
        let mut rooms = Vec::with_capacity(usize::from(SIZE) * usize::from(SIZE));
        for y in 0..SIZE {
            for x in 0..SIZE {
                let at = Position::new(x, y).expect("inside grid");
                let foe = if at == start() {
                    None
                } else if at == boss_room() {
                    Some(MonsterKind::BoneWarden)
                } else if rng.roll(10).expect("d10") <= 4 {
                    let pick = rng.roll(3).expect("d3") - 1;
                    Some(MonsterKind::ORDINARY[pick as usize])
                } else {
                    None
                };
                rooms.push(foe.map(|kind| Foe {
                    kind,
                    body: kind.spawn(),
                }));
            }
        }
        Self { rooms }
    }

    /// Build a dungeon from explicit rooms (row by row). Used when loading a save.
    pub fn from_rooms(rooms: Vec<Option<Foe>>) -> Option<Self> {
        (rooms.len() == usize::from(SIZE) * usize::from(SIZE)).then_some(Self { rooms })
    }

    /// All rooms, row by row.
    pub fn rooms(&self) -> &[Option<Foe>] {
        &self.rooms
    }

    pub fn foe(&self, at: Position) -> Option<&Foe> {
        self.rooms[index(at)].as_ref()
    }

    pub fn foe_mut(&mut self, at: Position) -> Option<&mut Foe> {
        self.rooms[index(at)].as_mut()
    }

    /// Remove a defeated foe from its room.
    pub fn clear(&mut self, at: Position) {
        self.rooms[index(at)] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{Dungeon, boss_room, start};
    use crate::dice::Rng;
    use crate::monsters::MonsterKind;

    #[test]
    fn same_seed_builds_the_same_dungeon() {
        assert_eq!(
            Dungeon::generate(&mut Rng::new(2026)),
            Dungeon::generate(&mut Rng::new(2026))
        );
    }

    #[test]
    fn start_is_empty_and_exactly_one_boss_waits_in_the_far_corner() {
        for seed in 0..200 {
            let dungeon = Dungeon::generate(&mut Rng::new(seed));
            assert!(dungeon.foe(start()).is_none());
            assert_eq!(
                dungeon.foe(boss_room()).unwrap().kind,
                MonsterKind::BoneWarden
            );
            let bosses = dungeon
                .rooms()
                .iter()
                .flatten()
                .filter(|foe| foe.kind == MonsterKind::BoneWarden)
                .count();
            assert_eq!(bosses, 1, "seed {seed}");
        }
    }

    #[test]
    fn clear_removes_a_foe_and_from_rooms_checks_size() {
        let mut dungeon = Dungeon::generate(&mut Rng::new(1));
        dungeon.clear(boss_room());
        assert!(dungeon.foe(boss_room()).is_none());
        assert!(Dungeon::from_rooms(vec![None; 24]).is_none());
        assert!(Dungeon::from_rooms(vec![None; 25]).is_some());
    }
}
