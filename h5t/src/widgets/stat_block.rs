use h5t_core::{monster::{Size, Speed, Type, Usage}, Monster};
use ratatui::{prelude::*, widgets::*};
use super::AbilityScores;

/// Creates a [`Paragraph`] widget for displaying the monster's name and type.
fn name_and_type_paragraph(monster: &'_ Monster) -> Paragraph<'_> {
    let size = match monster.size {
        Size::Tiny => "Tiny",
        Size::Small => "Small",
        Size::Medium => "Medium",
        Size::Large => "Large",
        Size::Huge => "Huge",
        Size::Gargantuan => "Gargantuan",
    };
    let r#type = match monster.r#type {
        Type::Aberration => "Aberration",
        Type::Beast => "Beast",
        Type::Celestial => "Celestial",
        Type::Construct => "Construct",
        Type::Dragon => "Dragon",
        Type::Elemental => "Elemental",
        Type::Fey => "Fey",
        Type::Fiend => "Fiend",
        Type::Giant => "Giant",
        Type::Humanoid => "Humanoid",
        Type::Monstrosity => "Monstrosity",
        Type::Ooze => "Ooze",
        Type::Plant => "Plant",
        Type::Undead => "Undead",
        Type::Other => "Other",
    };
    let subtype = if let Some(subtype) = &monster.subtype {
        format!(" ({})", subtype)
    } else {
        "".to_string()
    };

    Paragraph::new(vec![
        Span::styled(&monster.name, Modifier::BOLD).into(),
        Line::from(vec![
            Span::raw(size),
            Span::raw(" "),
            Span::raw(r#type),
            Span::raw(", "),
            Span::raw(&monster.alignment),
            Span::raw(subtype),
        ]).style(Modifier::ITALIC),
    ])
}

/// Creates a [`Table`] widget for displaying a monster's basic statistics.
fn basic_stats_table(monster: &'_ Monster) -> Table<'_> {
    /// Format's a speed value.
    fn fmt_speed(speed: &Speed) -> String {
        let mut parts = String::new();
        if let Some(speed) = &speed.walk {
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.burrow {
            parts.push_str("burrow ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.climb {
            parts.push_str("climb ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.fly {
            parts.push_str("fly ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        if let Some(speed) = &speed.swim {
            parts.push_str("swim ");
            parts.push_str(speed);
            parts.push_str(", ");
        }
        parts.pop(); // remove trailing comma
        parts.pop(); // remove trailing space
        parts
    }

    /// Formats a challenge rating.
    fn fmt_cr(cr: f32, xp: i32) -> String {
        let cr_value = if cr == 0.0 {
            "0".to_string()
        } else if cr < 1.0 {
            format!("1/{}", 1.0 / cr)
        } else {
            cr.to_string()
        };

        format!("{} ({} XP)", cr_value, xp)
    }

    Table::new(
        vec![
            Row::new(vec![
                Text::styled("Armor Class", Modifier::BOLD),
                Text::raw(monster.armor_class.value.to_string()),
            ]),
            Row::new(vec![
                Text::styled("Hit Points", Modifier::BOLD),
                Text::raw(format!("{} ({})", monster.hit_points, monster.hit_points_roll)),
            ]),
            Row::new(vec![
                Text::styled("Speed", Modifier::BOLD),
                Text::raw(fmt_speed(&monster.speed)),
            ]),
            Row::new(vec![
                Text::styled("Challenge", Modifier::BOLD),
                Text::raw(fmt_cr(monster.challenge_rating, monster.xp)),
            ]),
            Row::new(vec![
                Text::styled("Proficiency Bonus", Modifier::BOLD),
                Text::raw(format!("{:+}", monster.proficiency_bonus)),
            ]),
        ],
        vec![
            Constraint::Percentage(50), // stat name
            Constraint::Percentage(50), // stat value
        ],
    )
}

/// Creates a [`Paragraph`] widget for displaying a monster's traits.
fn traits_paragraph(monster: &'_ Monster) -> Paragraph<'_> {
    use itertools::Itertools;

    let text = monster
        .traits
        .iter()
        .map(|ability| {
            let constraint = match ability.usage {
                Usage::PerDay(count) => format!(" ({}/Day). ", count),
                Usage::RechargeAfterRest => " (Recharges after a Short or Long Rest). ".to_string(),
                Usage::RechargeAfterLongRest => " (Recharges after a Long Rest). ".to_string(),
                Usage::AtWill => ". ".to_string(),
            };
            Line::from(vec![
                Span::styled(&ability.name, Modifier::BOLD | Modifier::ITALIC),
                Span::styled(constraint, Modifier::BOLD | Modifier::ITALIC),
                Span::raw(&ability.desc),
            ])
        })
        .intersperse(Line::raw(""))
        .collect::<Vec<_>>();
    Paragraph::new(text)
        .wrap(Wrap { trim: true })
}

/// A widget for displaying a monster's stat block.
#[derive(Debug)]
pub struct StatBlock<'a> {
    /// The monster to display.
    pub monster: &'a Monster,
}

impl<'a> StatBlock<'a> {
    /// Create a new [`StatBlock`] widget.
    pub fn new(monster: &'a Monster) -> Self {
        Self { monster }
    }
}

impl<'a> Widget for StatBlock<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .title("Monster Stat Block")
            .render(area, buf);

        let layout = Layout::vertical([
            Constraint::Min(2), // name and type
            Constraint::Min(4), // basic stats
            Constraint::Min(6), // ability scores
            Constraint::Min(1), // traits
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(area);
        let [
            name,
            basic_stats,
            ability_scores,
            traits
        ] = [layout[0], layout[1], layout[2], layout[3]];

        name_and_type_paragraph(self.monster).render(name, buf);
        Widget::render(basic_stats_table(self.monster), basic_stats, buf);
        AbilityScores::from(self.monster).render(ability_scores, buf);
        traits_paragraph(self.monster).render(traits, buf);
    }
}
