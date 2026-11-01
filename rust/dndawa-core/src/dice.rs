//! Deterministic dice: a small seedable generator (SplitMix64) and unbiased die rolls.

/// Seedable pseudo-random generator. The same seed always gives the same rolls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

/// Why a roll was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiceError {
    /// A die must have at least one side.
    ZeroSides,
}

impl std::fmt::Display for DiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroSides => f.write_str("a die needs at least one side"),
        }
    }
}

impl std::error::Error for DiceError {}

impl Rng {
    /// Create a generator from a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Internal state, so a game can be saved and resumed exactly.
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Next raw 64-bit value (SplitMix64).
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Roll one die with `sides` sides: a value in `1..=sides`, without modulo bias.
    pub fn roll(&mut self, sides: u32) -> Result<u32, DiceError> {
        if sides == 0 {
            return Err(DiceError::ZeroSides);
        }
        let sides = u64::from(sides);
        let zone = u64::MAX - (u64::MAX % sides);
        loop {
            let value = self.next_u64();
            if value < zone {
                // value % sides < sides <= u32::MAX, so the conversion cannot fail.
                return Ok(u32::try_from(value % sides).expect("below sides") + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DiceError, Rng};

    #[test]
    fn same_seed_gives_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.roll(20), b.roll(20));
        }
    }

    #[test]
    fn rolls_stay_in_range_and_hit_every_face() {
        let mut rng = Rng::new(7);
        let mut seen = [false; 20];
        for _ in 0..10_000 {
            let value = rng.roll(20).unwrap();
            assert!((1..=20).contains(&value));
            seen[(value - 1) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn one_sided_die_always_rolls_one() {
        let mut rng = Rng::new(1);
        assert!((0..50).all(|_| rng.roll(1) == Ok(1)));
    }

    #[test]
    fn zero_sides_is_refused_without_advancing_the_generator() {
        let mut rng = Rng::new(9);
        let before = rng.clone();
        assert_eq!(rng.roll(0), Err(DiceError::ZeroSides));
        assert_eq!(rng, before);
    }
}
