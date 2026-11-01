//! Stat cards: a small framed card for the hero or a foe.
//!
//! ```text
//! ┌─ Hero ───────────────────────┐
//! │ HP 12/12 ██████████          │
//! │ AC 16 · ATK +5 · DMG 1d8+3   │
//! └──────────────────────────────┘
//! ```

use crate::combatant::Combatant;

/// Width of every card, frame included, in characters.
pub const CARD_WIDTH: usize = 32;

/// Cells in the hit-point bar.
const BAR: u64 = 10;

/// The four lines of a card titled `title` for `body`.
pub fn card(title: &str, body: &Combatant) -> [String; 4] {
    let inner = CARD_WIDTH - 2;
    let title: String = title.chars().take(inner - 4).collect();
    let rule = "─".repeat(inner - 3 - title.chars().count());
    [
        format!("┌─ {title} {rule}┐"),
        format!(
            "│{}│",
            fit(
                &format!(" HP {}/{} {}", body.hp(), body.max_hp(), bar(body)),
                inner
            )
        ),
        format!(
            "│{}│",
            fit(
                &format!(
                    " AC {} · ATK {:+} · DMG {}",
                    body.armor_class, body.attack_bonus, body.damage
                ),
                inner
            )
        ),
        format!("└{}┘", "─".repeat(inner)),
    ]
}

/// Hit points as filled and empty cells; any living fighter shows at least one.
fn bar(body: &Combatant) -> String {
    let (hp, max) = (u64::from(body.hp()), u64::from(body.max_hp()));
    let filled = (hp * BAR).div_ceil(max).min(BAR);
    let empty = BAR - filled;
    format!(
        "{}{}",
        "█".repeat(filled as usize),
        "░".repeat(empty as usize)
    )
}

/// Cut or pad `text` to exactly `width` characters.
fn fit(text: &str, width: usize) -> String {
    let cut: String = text.chars().take(width).collect();
    let pad = width - cut.chars().count();
    format!("{cut}{}", " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::{CARD_WIDTH, card};
    use crate::game::new_hero;
    use crate::monsters::MonsterKind;

    fn width_ok(lines: &[String]) {
        for line in lines {
            assert_eq!(line.chars().count(), CARD_WIDTH, "{line}");
        }
    }

    #[test]
    fn hero_card_shows_hp_armor_attack_and_damage() {
        let lines = card("Hero", &new_hero());
        width_ok(&lines);
        assert!(lines[0].starts_with("┌─ Hero ─"));
        assert!(lines[1].contains("HP 12/12 ██████████"), "{}", lines[1]);
        assert!(
            lines[2].contains("AC 16 · ATK +5 · DMG 1d8+3"),
            "{}",
            lines[2]
        );
        assert!(lines[3].starts_with('└') && lines[3].ends_with('┘'));
    }

    #[test]
    fn bar_follows_hit_points() {
        let mut hero = new_hero();
        hero.take_damage(6);
        assert!(card("Hero", &hero)[1].contains("HP 6/12 █████░░░░░"));
        hero.take_damage(5);
        assert!(card("Hero", &hero)[1].contains("HP 1/12 █░░░░░░░░░"));
        hero.take_damage(1);
        assert!(card("Hero", &hero)[1].contains("HP 0/12 ░░░░░░░░░░"));
    }

    #[test]
    fn every_foe_card_fits_and_long_titles_are_cut() {
        for kind in MonsterKind::ORDINARY
            .into_iter()
            .chain([MonsterKind::BoneWarden])
        {
            let body = kind.spawn();
            let lines = card(&body.name, &body);
            width_ok(&lines);
            assert!(lines[2].contains(&format!("DMG {}", body.damage)));
        }
        width_ok(&card(&"x".repeat(80), &new_hero()));
    }
}
