//! Optional terminal colours (ANSI escape codes), only when the player asks with
//! `--color`. The game never colours by default, so `NO_COLOR` has nothing to turn off;
//! an explicit `--color` is honoured even when it is set, as no-color.org recommends for
//! per-command flags (<https://no-color.org/>).

const RESET: &str = "\x1b[0m";

/// One character in colour.
fn wrap(code: &str, c: char) -> String {
    format!("\x1b[{code}m{c}{RESET}")
}

/// Colour one line of the board: hero yellow, boss red, foes magenta, fog dim.
/// Only board lines go through here, so the letters can only be tokens.
pub fn paint_board(line: &str) -> String {
    line.chars()
        .map(|c| match c {
            '@' => wrap("1;33", c),
            'B' => wrap("1;31", c),
            'r' | 's' | 'g' => wrap("35", c),
            '?' => wrap("2", c),
            _ => c.to_string(),
        })
        .collect()
}

/// Colour one line beside the board (cards, legend): only the hit-point bar, so names
/// and the legend keep their own letters plain.
pub fn paint_side(line: &str) -> String {
    line.chars()
        .map(|c| match c {
            '█' => wrap("32", c),
            '░' => wrap("2", c),
            _ => c.to_string(),
        })
        .collect()
}

/// The `◄` pointer on the hero's row, in yellow.
pub fn pointer() -> String {
    wrap("1;33", '◄')
}

/// Remove colour codes again (used by tests and width checks).
pub fn strip(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for end in chars.by_ref() {
                if end == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{paint_board, paint_side, pointer, strip};

    #[test]
    fn the_board_colours_its_tokens() {
        let line = "│ · │@ g│ ? │ ? │ B │";
        let painted = paint_board(line);
        assert_eq!(strip(&painted), line);
        for token in ["\x1b[1;33m@", "\x1b[35mg", "\x1b[2m?", "\x1b[1;31mB"] {
            assert!(painted.contains(token), "{token:?} in {painted:?}");
        }
    }

    #[test]
    fn the_side_colours_only_the_bar_never_letters() {
        let line = "│ HP 7/7 ██░░ Target: Bone Warden · @ ? B r s g │";
        let painted = paint_side(line);
        assert_eq!(strip(&painted), line);
        assert!(painted.contains("\x1b[32m█") && painted.contains("\x1b[2m░"));
        assert!(painted.contains("Target: Bone Warden · @ ? B r s g"));
        assert_eq!(strip(&pointer()), "◄");
    }
}
