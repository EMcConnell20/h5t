use h5t_core::{
    ability::{Modifier as AbilityModifier, Score},
    Ability,
    Combatant,
    CombatantKind,
    Monster,
    score_to_modifier,
};
use ratatui::{prelude::*, widgets::*};

/// A widget to display a table of ability scores.
#[derive(Debug)]
pub struct AbilityScores {
    /// The ability scores to display.
    scores: Ability<Score>,

    /// The saving throw proficiencies.
    ///
    /// This is used to calculate saving throw modifiers. If this is `None`, the ability modifier
    /// is used instead.
    proficiencies: Ability<Option<AbilityModifier>>,
}

impl AbilityScores {
    /// Create a new [`AbilityScores`] widget from a [`Combatant`].
    pub fn new(combatant: &Combatant) -> Self {
        match &combatant.kind {
            CombatantKind::Monster(monster) => Self::from(monster),
        }
    }
}

impl From<&Monster> for AbilityScores {
    fn from(monster: &Monster) -> Self {
        Self {
            scores: monster.scores,
            proficiencies: monster.proficiencies.saving_throws,
        }
    }
}

impl Widget for AbilityScores {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (str, dex, con, int, wis, cha) = (
            self.scores.strength,
            self.scores.dexterity,
            self.scores.constitution,
            self.scores.intelligence,
            self.scores.wisdom,
            self.scores.charisma,
        );
        let (str_save, dex_save, con_save, int_save, wis_save, cha_save) = (
            self.proficiencies.strength,
            self.proficiencies.dexterity,
            self.proficiencies.constitution,
            self.proficiencies.intelligence,
            self.proficiencies.wisdom,
            self.proficiencies.charisma,
        );

        /// Helper to build a row for the ability scores table.
        fn row(odd: bool, ability: &'_ str, score: i32, save: Option<i32>) -> Row<'_> {
            // more green for high scores, more red for low scores
            // 0: (255, 0, 0)
            // 10: (255, 255, 255)
            // 20: (0, 255, 0)
            fn score_to_color(score: i32) -> Color {
                Color::Rgb(
                    (510.0 - 255.0 / 10.0 * score as f32).min(255.0) as u8,
                    (255.0 / 10.0 * score as f32).min(255.0) as u8,
                    (255.0 - (255.0 / 10.0 * score as f32 - 255.0).abs()).max(0.0) as u8,
                )
            }

            let modifier = score_to_modifier(score);
            let main_color = score_to_color(score);

            // compute color for save modifier by mocking an increased ability score
            let save_color = score_to_color(score + 2 * (save.unwrap_or(modifier) - modifier));

            Row::new(vec![
                Text::styled(ability, Modifier::BOLD),
                Text::styled(score.to_string(), main_color),
                Text::styled(format!("{:+}", modifier), main_color),
                Text::styled(format!("{:+}", save.unwrap_or(modifier)), save_color),
            ])
                .style(Style::default().bg(if odd { Color::DarkGray } else { Color::Black }))
        }

        let widget = Table::new(
            vec![
                row(false, "STR", str, str_save),
                row(true, "DEX", dex, dex_save),
                row(false, "CON", con, con_save),
                row(true, "INT", int, int_save),
                row(false, "WIS", wis, wis_save),
                row(true, "CHA", cha, cha_save),
            ],
            vec![
                Constraint::Length(7), // ability abbreviation
                Constraint::Length(5), // ability score
                Constraint::Length(4), // modifier
                Constraint::Length(4), // saving throw modifier
            ],
        )
            .header(Row::new(vec![
                Text::from("Ability"),
                Text::from("Score"),
                Text::from("Mod"),
                Text::from("Save"),
            ]).bold());
        Widget::render(widget, area, buf);
    }
}
