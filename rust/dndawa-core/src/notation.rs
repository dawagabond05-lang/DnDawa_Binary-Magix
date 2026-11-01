//! Dice notation such as `d20`, `2d6+3` or `1d8-1`.

use crate::dice::Rng;

/// Most dice allowed in one expression, to keep rolls bounded.
pub const MAX_COUNT: u32 = 100;
/// Most sides allowed on one die.
pub const MAX_SIDES: u32 = 1000;

/// A parsed dice expression: `count` dice of `sides` sides, plus `modifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dice {
    pub count: u32,
    pub sides: u32,
    pub modifier: i32,
}

/// Why a dice expression could not be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiceError {
    /// No `d` separating count and sides.
    MissingD,
    /// A number is missing, malformed or out of range.
    BadNumber,
    /// Count is 0 or above [`MAX_COUNT`].
    CountOutOfRange,
    /// Sides is 0 or above [`MAX_SIDES`].
    SidesOutOfRange,
}

impl std::fmt::Display for ParseDiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingD => "dice notation needs a 'd', like 2d6",
            Self::BadNumber => "dice notation has a missing or invalid number",
            Self::CountOutOfRange => "dice count must be between 1 and 100",
            Self::SidesOutOfRange => "dice sides must be between 1 and 1000",
        })
    }
}

impl std::error::Error for ParseDiceError {}

impl std::str::FromStr for Dice {
    type Err = ParseDiceError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        let (count, rest) = text.split_once('d').ok_or(ParseDiceError::MissingD)?;
        let count = if count.is_empty() {
            1
        } else {
            parse_digits(count)?
        };
        let (sides, modifier) = match rest.find(['+', '-']) {
            Some(at) => {
                let (sides, signed) = rest.split_at(at);
                let magnitude = i32::try_from(parse_digits(&signed[1..])?)
                    .map_err(|_| ParseDiceError::BadNumber)?;
                let modifier = if signed.starts_with('-') {
                    -magnitude
                } else {
                    magnitude
                };
                (parse_digits(sides)?, modifier)
            }
            None => (parse_digits(rest)?, 0),
        };
        if !(1..=MAX_COUNT).contains(&count) {
            return Err(ParseDiceError::CountOutOfRange);
        }
        if !(1..=MAX_SIDES).contains(&sides) {
            return Err(ParseDiceError::SidesOutOfRange);
        }
        Ok(Self {
            count,
            sides,
            modifier,
        })
    }
}

/// Parse plain ASCII digits only (no sign, no spaces).
fn parse_digits(text: &str) -> Result<u32, ParseDiceError> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ParseDiceError::BadNumber);
    }
    text.parse().map_err(|_| ParseDiceError::BadNumber)
}

/// Writes the short form, such as `1d8+3`, `2d6` or `1d4-1`; it parses back the same.
impl std::fmt::Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)?;
        if self.modifier > 0 {
            write!(f, "+{}", self.modifier)?;
        } else if self.modifier < 0 {
            write!(f, "{}", self.modifier)?;
        }
        Ok(())
    }
}

impl Dice {
    /// Roll all dice and add the modifier. Fits easily in `i64` given the limits.
    pub fn roll(&self, rng: &mut Rng) -> i64 {
        let total: i64 = (0..self.count)
            .map(|_| i64::from(rng.roll(self.sides).expect("sides >= 1 by construction")))
            .sum();
        total + i64::from(self.modifier)
    }
}

#[cfg(test)]
mod tests {
    use super::{Dice, ParseDiceError};
    use crate::dice::Rng;

    fn parse(text: &str) -> Result<Dice, ParseDiceError> {
        text.parse()
    }

    #[test]
    fn parses_common_forms() {
        assert_eq!(
            parse("d20").unwrap(),
            Dice {
                count: 1,
                sides: 20,
                modifier: 0
            }
        );
        assert_eq!(
            parse("2d6+3").unwrap(),
            Dice {
                count: 2,
                sides: 6,
                modifier: 3
            }
        );
        assert_eq!(
            parse(" 1d8-1 ").unwrap(),
            Dice {
                count: 1,
                sides: 8,
                modifier: -1
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(parse("20"), Err(ParseDiceError::MissingD));
        assert_eq!(parse("2d"), Err(ParseDiceError::BadNumber));
        assert_eq!(parse("xd6"), Err(ParseDiceError::BadNumber));
        assert_eq!(parse("2d6+"), Err(ParseDiceError::BadNumber));
        assert_eq!(parse("2d6+-1"), Err(ParseDiceError::BadNumber));
        assert_eq!(parse("0d6"), Err(ParseDiceError::CountOutOfRange));
        assert_eq!(parse("101d6"), Err(ParseDiceError::CountOutOfRange));
        assert_eq!(parse("1d0"), Err(ParseDiceError::SidesOutOfRange));
        assert_eq!(parse("99999999999d6"), Err(ParseDiceError::BadNumber));
    }

    #[test]
    fn displays_the_short_form_and_parses_back() {
        for (text, shown) in [("d20", "1d20"), ("2d6+3", "2d6+3"), ("1d8-1", "1d8-1")] {
            let dice = parse(text).unwrap();
            assert_eq!(dice.to_string(), shown);
            assert_eq!(parse(shown), Ok(dice));
        }
    }

    #[test]
    fn roll_stays_within_bounds() {
        let dice = parse("2d6+3").unwrap();
        let mut rng = Rng::new(5);
        for _ in 0..1000 {
            let total = dice.roll(&mut rng);
            assert!((5..=15).contains(&total));
        }
    }
}
