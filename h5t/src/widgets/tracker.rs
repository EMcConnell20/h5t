use crate::{ui::LabelModeState, widgets::{CompactConditions, HitPoints}};
use h5t_core::{Action, Combatant, Tracker as CoreTracker};
use ratatui::{prelude::*, widgets::*};

/// Mix two RGB colors together.
fn mix_colors(color1: (u8, u8, u8), color2: (u8, u8, u8)) -> (u8, u8, u8) {
    let f = |n1: u8, n2: u8| {
        let n1 = (255 - n1) as f32;
        let n2 = (255 - n2) as f32;
        (n1.powi(2) + n2.powi(2)).sqrt() / 2.0
    };
    (
        f(color1.0, color2.0) as u8,
        f(color1.1, color2.1) as u8,
        f(color1.2, color2.2) as u8,
    )
}

/// Creates a [`Line`] widget for displaying a list of actions.
fn action_line(actions: Action) -> Line<'static> {
    /// Format multiple actions in a compact way (e.g. `Ax4,R`).
    fn fmt_action(label: &str, count: u32) -> String {
        if count <= 3 {
            label.repeat(count as usize)
        } else {
            format!("{}x{}", label, count)
        }
    }

    let mut spans = Vec::new();
    if actions.actions > 0 {
        spans.push(Span::styled(fmt_action("A", actions.actions), Color::Green));
        spans.push(Span::raw(","));
    }
    if actions.bonus_actions > 0 {
        spans.push(Span::styled(fmt_action("BA", actions.bonus_actions), Color::Rgb(255, 165, 0)));
        spans.push(Span::raw(","));
    }
    if actions.reactions > 0 {
        spans.push(Span::styled(fmt_action("R", actions.reactions), Color::Magenta));
        spans.push(Span::raw(","));
    }
    spans.pop(); // remove the last comma
    Line::from(spans)
}

/// Creates a [`Table`] widget for displaying the combatants in the tracker.
fn combatant_table<'a>(widget: &'a Tracker) -> Table<'a> {
    /// Builds a table [`Row`] for a combatant.
    fn combatant_row(label: Option<char>, combatant: &'_ Combatant) -> Row<'_> {
        let label_text = label
            .map(|l| Text::from(format!("{}", l)).bold())
            .unwrap_or_default();
        Row::new([
            label_text,
            Text::from(combatant.name()),
            action_line(combatant.actions).into(),
            HitPoints::new(combatant).line().into(),
            CompactConditions::new(combatant).line().into(),
        ])
    }

    Table::new(
        widget.tracker.combatants.iter()
            .enumerate()
            .map(|(i, combatant)| {
                let is_current_turn = i == widget.tracker.turn;
                let label = widget.label_state.labels.get_by_right(&i).copied();
                let is_label_selected = widget.label_state.selected.contains(&label.unwrap_or_default());

                let row = combatant_row(label, combatant);
                let mut style = Style::default();
                if is_label_selected {
                    style = style.bold();
                }

                let mut bg_color = None;
                if combatant.hit_points <= 0 {
                    bg_color = bg_color
                        .map(|current| mix_colors((255, 0, 0), current))
                        .or(Some((100, 0, 0)));
                }
                if is_current_turn {
                    bg_color = bg_color
                        .map(|current| mix_colors((0, 48, 130), current))
                        .or(Some((0, 48, 130)));
                }
                if is_label_selected {
                    bg_color = bg_color
                        .map(|current| mix_colors((128, 85, 0), current))
                        .or(Some((128, 85, 0)));
                }

                let bg_color = bg_color.map(|bg| Color::Rgb(bg.0, bg.1, bg.2)).unwrap_or(Color::Reset);
                style = style.bg(bg_color);

                row.style(style)
            }),
        [
            Constraint::Length(2), // label mode
            Constraint::Fill(2),   // name
            Constraint::Fill(1),   // actions
            Constraint::Fill(1),   // hp / max hp
            Constraint::Fill(1),   // conditions
        ],
    )
        .header(Row::new([
            Text::raw(""),
            Text::from("Name").centered(),
            Text::from("Actions").centered(),
            Text::from("HP / Max HP").centered(),
            Text::from("Conditions").centered(),
        ]).bold())
}

/// A widget to render the initiative tracker's state.
#[derive(Debug)]
pub struct Tracker<'a> {
    /// The tracker to display.
    pub tracker: &'a CoreTracker,

    /// State for label mode.
    pub label_state: LabelModeState,
}

impl<'a> Tracker<'a> {
    /// Create a new [`Tracker`] widget.
    pub fn new(tracker: &'a CoreTracker) -> Self {
        Self { tracker, label_state: LabelModeState::default() }
    }

    /// Create a new [`Tracker`] widget with the given labels.
    pub fn with_labels(tracker: &'a CoreTracker, label: LabelModeState) -> Self {
        Self { tracker, label_state: label }
    }
}

/// Returns the maximum number of combatants that can be displayed in the tracker widget, given the
/// size of the widget.
pub(crate) fn max_combatants(size: Size) -> usize {
    size.height as usize - 6 // 2 for upper and lower borders, 4 for header, spacing, etc.
}

impl<'a> Widget for Tracker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw bordered box for the tracker
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White))
            .title("Initiative Tracker")
            .render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(2), // round and turn
            Constraint::Fill(1),
        ])
            .horizontal_margin(2)
            .vertical_margin(1) // avoid the border
            .spacing(1)
            .split(area);
        let [round_and_turn, combatants] = [layout[0], layout[1]];

        let text = vec![
            Line::styled(format!("Round: {}", self.tracker.round + 1), Modifier::BOLD),
            Line::styled(
                format!("Turn: {}/{}", self.tracker.turn + 1, self.tracker.combatants.len()),
                Modifier::BOLD
            ),
        ];
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .render(round_and_turn, buf);

        Widget::render(combatant_table(&self), combatants, buf);
    }
}
