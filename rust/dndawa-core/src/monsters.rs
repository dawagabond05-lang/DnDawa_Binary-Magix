//! The three monster types and the final boss.
//!
//! Rat, Skeleton and Goblin use SRD 5.1 statblocks (`10_Monsters/Monsters_Each/`
//! `Giant_Rat_(Creature).md`, `Skeleton.md`, `Goblin.md`: AC, average HP, attack bonus
//! and hit dice of the first attack). The Bone Warden is HOMEBREW (PENDING_DAWA).

use crate::combatant::Combatant;
use crate::notation::Dice;

/// Every kind of foe in the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterKind {
    GiantRat,
    Skeleton,
    Goblin,
    /// Final boss. HOMEBREW stats.
    BoneWarden,
}

impl MonsterKind {
    /// The three ordinary monster types (the boss is not among them).
    pub const ORDINARY: [MonsterKind; 3] = [Self::GiantRat, Self::Skeleton, Self::Goblin];

    /// Stable short code, used in save files.
    pub fn code(self) -> &'static str {
        match self {
            Self::GiantRat => "rat",
            Self::Skeleton => "skeleton",
            Self::Goblin => "goblin",
            Self::BoneWarden => "boss",
        }
    }

    /// Inverse of [`MonsterKind::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "rat" => Some(Self::GiantRat),
            "skeleton" => Some(Self::Skeleton),
            "goblin" => Some(Self::Goblin),
            "boss" => Some(Self::BoneWarden),
            _ => None,
        }
    }

    /// A fresh combatant of this kind at full health.
    pub fn spawn(self) -> Combatant {
        let (name, hp, ac, bonus, damage) = match self {
            Self::GiantRat => ("Giant Rat", 7, 12, 4, dice(1, 4, 2)),
            Self::Skeleton => ("Skeleton", 13, 13, 4, dice(1, 6, 2)),
            Self::Goblin => ("Goblin", 7, 15, 4, dice(1, 6, 2)),
            Self::BoneWarden => ("Bone Warden", 45, 15, 5, dice(1, 10, 3)),
        };
        Combatant::new(name, hp, ac, bonus, damage)
    }
}

const fn dice(count: u32, sides: u32, modifier: i32) -> Dice {
    Dice {
        count,
        sides,
        modifier,
    }
}

#[cfg(test)]
mod tests {
    use super::MonsterKind;

    #[test]
    fn srd_statblocks_match_the_scrolls() {
        let rat = MonsterKind::GiantRat.spawn();
        assert_eq!(
            (rat.max_hp(), rat.armor_class, rat.attack_bonus),
            (7, 12, 4)
        );
        let skeleton = MonsterKind::Skeleton.spawn();
        assert_eq!((skeleton.max_hp(), skeleton.armor_class), (13, 13));
        let goblin = MonsterKind::Goblin.spawn();
        assert_eq!((goblin.max_hp(), goblin.armor_class), (7, 15));
    }

    #[test]
    fn codes_round_trip_and_unknown_is_rejected() {
        for kind in MonsterKind::ORDINARY
            .into_iter()
            .chain([MonsterKind::BoneWarden])
        {
            assert_eq!(MonsterKind::from_code(kind.code()), Some(kind));
        }
        assert_eq!(MonsterKind::from_code("dragon"), None);
    }

    #[test]
    fn boss_is_not_an_ordinary_monster_and_is_tougher() {
        assert!(!MonsterKind::ORDINARY.contains(&MonsterKind::BoneWarden));
        assert!(MonsterKind::BoneWarden.spawn().max_hp() > MonsterKind::Skeleton.spawn().max_hp());
    }
}
