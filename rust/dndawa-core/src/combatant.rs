//! Anything that fights: hit points, armor class, attack bonus and damage dice.

use crate::notation::Dice;

/// A fighter. `hp` never exceeds `max_hp` and never goes below 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combatant {
    pub name: String,
    hp: u32,
    max_hp: u32,
    pub armor_class: u8,
    pub attack_bonus: i8,
    pub damage: Dice,
}

impl Combatant {
    /// Create a combatant at full health. `max_hp` must be at least 1.
    pub fn new(
        name: impl Into<String>,
        max_hp: u32,
        armor_class: u8,
        attack_bonus: i8,
        damage: Dice,
    ) -> Self {
        let max_hp = max_hp.max(1);
        Self {
            name: name.into(),
            hp: max_hp,
            max_hp,
            armor_class,
            attack_bonus,
            damage,
        }
    }

    pub fn hp(&self) -> u32 {
        self.hp
    }

    pub fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// Lose hit points, stopping at 0. Returns the hit points actually lost.
    pub fn take_damage(&mut self, amount: u32) -> u32 {
        let lost = amount.min(self.hp);
        self.hp -= lost;
        lost
    }

    /// Regain hit points, stopping at `max_hp`. Returns the hit points actually regained.
    pub fn heal(&mut self, amount: u32) -> u32 {
        let gained = amount.min(self.max_hp - self.hp);
        self.hp += gained;
        gained
    }

    /// Restore an exact hit-point value (used when loading a save). Values are clamped.
    pub fn set_hp(&mut self, hp: u32) {
        self.hp = hp.min(self.max_hp);
    }
}

#[cfg(test)]
mod tests {
    use super::Combatant;

    fn fighter() -> Combatant {
        Combatant::new("Hero", 12, 15, 5, "1d8+3".parse().unwrap())
    }

    #[test]
    fn starts_at_full_health() {
        let hero = fighter();
        assert_eq!((hero.hp(), hero.max_hp()), (12, 12));
        assert!(hero.is_alive());
    }

    #[test]
    fn damage_stops_at_zero() {
        let mut hero = fighter();
        assert_eq!(hero.take_damage(5), 5);
        assert_eq!(hero.take_damage(100), 7);
        assert_eq!(hero.hp(), 0);
        assert!(!hero.is_alive());
    }

    #[test]
    fn healing_stops_at_max() {
        let mut hero = fighter();
        hero.take_damage(4);
        assert_eq!(hero.heal(10), 4);
        assert_eq!(hero.hp(), 12);
    }

    #[test]
    fn zero_max_hp_is_raised_to_one_and_set_hp_is_clamped() {
        let mut ghost = Combatant::new("Ghost", 0, 10, 0, "1d4".parse().unwrap());
        assert_eq!(ghost.max_hp(), 1);
        ghost.set_hp(99);
        assert_eq!(ghost.hp(), 1);
    }
}
