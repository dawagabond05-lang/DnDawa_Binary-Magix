//! Attack rolls: d20 + attack bonus against armor class
//! (SRD 5.1, `06_Gameplay/Order_of_Combat.md:241`). A natural 1 always misses;
//! a natural 20 always hits and is a critical hit (damage dice rolled twice).

use crate::combatant::Combatant;
use crate::dice::Rng;

/// How an attack roll landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Miss,
    Hit,
    Critical,
}

/// Result of one attack, including the natural d20 for the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackOutcome {
    pub natural: u32,
    pub kind: HitKind,
    pub damage: u32,
}

/// Pure hit check. `natural` is the d20 face (1..=20).
pub fn hit_kind(natural: u32, attack_bonus: i8, target_ac: u8) -> HitKind {
    match natural {
        1 => HitKind::Miss,
        20 => HitKind::Critical,
        _ => {
            let total = i64::from(natural) + i64::from(attack_bonus);
            if total >= i64::from(target_ac) {
                HitKind::Hit
            } else {
                HitKind::Miss
            }
        }
    }
}

/// Roll an attack by `attacker` against `target_ac`. Damage is never negative.
pub fn attack(attacker: &Combatant, target_ac: u8, rng: &mut Rng) -> AttackOutcome {
    let natural = rng.roll(20).expect("d20 has sides");
    let kind = hit_kind(natural, attacker.attack_bonus, target_ac);
    let dice = attacker.damage;
    let damage = match kind {
        HitKind::Miss => 0,
        HitKind::Hit => dice.roll(rng),
        // Critical: roll the damage dice twice, add the modifier once.
        HitKind::Critical => dice.roll(rng) + dice.roll(rng) - i64::from(dice.modifier),
    };
    AttackOutcome {
        natural,
        kind,
        damage: u32::try_from(damage.max(0)).unwrap_or(u32::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::{HitKind, attack, hit_kind};
    use crate::combatant::Combatant;
    use crate::dice::Rng;

    #[test]
    fn natural_one_misses_and_natural_twenty_crits_whatever_the_numbers() {
        assert_eq!(hit_kind(1, 100, 0), HitKind::Miss);
        assert_eq!(hit_kind(20, -100, 255), HitKind::Critical);
    }

    #[test]
    fn total_meeting_ac_hits_and_below_misses() {
        assert_eq!(hit_kind(10, 5, 15), HitKind::Hit);
        assert_eq!(hit_kind(9, 5, 15), HitKind::Miss);
        assert_eq!(hit_kind(19, -5, 14), HitKind::Hit);
    }

    #[test]
    fn misses_deal_no_damage_and_hits_stay_in_dice_range() {
        let goblin = Combatant::new("Goblin", 7, 15, 4, "1d6+2".parse().unwrap());
        let mut rng = Rng::new(11);
        for _ in 0..2000 {
            let out = attack(&goblin, 13, &mut rng);
            match out.kind {
                HitKind::Miss => assert_eq!(out.damage, 0),
                HitKind::Hit => assert!((3..=8).contains(&out.damage)),
                HitKind::Critical => assert!((4..=14).contains(&out.damage)),
            }
            assert!((1..=20).contains(&out.natural));
        }
    }

    #[test]
    fn negative_damage_modifier_never_heals() {
        let weak = Combatant::new("Rat", 1, 10, 0, "1d1-5".parse().unwrap());
        let mut rng = Rng::new(3);
        assert!((0..500).all(|_| attack(&weak, 0, &mut rng).damage == 0));
    }
}
