//! Ability scores. Rule: modifier = floor((score - 10) / 2)
//! (SRD 5.1, `06_Gameplay/Using_Ability_Scores.md:16`).

/// Modifier for an ability score. Scores 1..=30 give -5..=+10.
pub fn ability_modifier(score: u8) -> i8 {
    // (score - 10) fits in i16; div_euclid rounds toward negative infinity (floor).
    let modifier = (i16::from(score) - 10).div_euclid(2);
    i8::try_from(modifier).expect("u8 score gives a modifier in -5..=122")
}

#[cfg(test)]
mod tests {
    use super::ability_modifier;

    #[test]
    fn matches_the_srd_table() {
        let table = [
            (1, -5),
            (3, -4),
            (8, -1),
            (9, -1),
            (10, 0),
            (11, 0),
            (12, 1),
            (20, 5),
            (30, 10),
        ];
        for (score, expected) in table {
            assert_eq!(ability_modifier(score), expected, "score {score}");
        }
    }

    #[test]
    fn whole_u8_range_does_not_panic() {
        for score in 0..=u8::MAX {
            let _ = ability_modifier(score);
        }
        assert_eq!(ability_modifier(0), -5);
        assert_eq!(ability_modifier(u8::MAX), 122);
    }
}
