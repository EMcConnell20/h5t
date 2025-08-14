// -- Imports -- //

use crate::ui::{Page, LabelSelection};

use h5t_core::Action;
use h5t_core::Tracker as CoreTracker;

use ratatui::prelude::*;
use ratatui::widgets::*;

// -- Constants -- //

const DIVIDER_CHARACTER: &str = " | ";
const ACTION_COLOR: Color = Color::Green;
const BONUS_ACTION_COLOR: Color = Color::Rgb(255, 165, 0);

// -- Exports -- //

/// Returns the maximum number of combatants that can be displayed in the tracker widget.
pub(crate) fn max_combatants_visible(widget_size: Size) -> usize {
	// 2 Lines for upper and lower borders
	// 4 Lines for header, spacing, etc...
	// maximum of 32 combatants per page
	(widget_size.height as usize).saturating_sub(6).min(32)
}

#[derive(Copy, Clone, Debug)]
pub struct TrackerWidget<'a> {
	tracker: &'a CoreTracker,
	page: Option<&'a Page>,
	draw_labels: bool,
}

impl<'a> TrackerWidget<'a> {
	pub fn new(tracker: &'a CoreTracker, page: Option<&'a Page>, draw_labels: bool) -> Self {
		Self { tracker, page, draw_labels }
	}
}

impl<'a> Widget for TrackerWidget<'a> {
	fn render(self, area: Rect, buf: &mut Buffer)
	where
		Self: Sized
	{
		Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(Style::default().fg(Color::White))
			.title("Initiative Tracker")
			.render(area, buf);
		
		let layout = Layout::vertical([
			Constraint::Length(3), // round and turn
			Constraint::Fill(1),
		])
			.horizontal_margin(2)
			.vertical_margin(1) // avoid the border
			.spacing(1)
			.split(area);
		
		let [round_and_turn, combatants] = [layout[0], layout[1]];
		
		let page_number = self.page.map(|p| p.get_id()).unwrap_or(0);
		
		let text = vec![
			Line::styled(format!("Page: {}", page_number + 1), Modifier::BOLD),
			Line::styled(format!("Round: {}", self.tracker.round + 1), Modifier::BOLD),
			Line::styled(
				format!("Turn: {}/{}", self.tracker.turn + 1, self.tracker.combatants.len()),
				Modifier::BOLD
			),
		];
		
		Paragraph::new(text)
			.wrap(Wrap { trim: true })
			.render(round_and_turn, buf);
		
		Widget::render(make_combat_table(self), combatants, buf);
	}
}

// -- Private Functions -- //

/// Creates a [`Line`] widget for displaying a list of actions.
fn action_line(actions: Action) -> Line<'static> {
	use utility_functions::fmt_action;
	
	let mut spans = Vec::new();
	
	if actions.actions > 0 {
		spans.push(Span::styled(fmt_action("A", actions.actions), ACTION_COLOR));
		spans.push(Span::raw(DIVIDER_CHARACTER));
	}
	
	if actions.bonus_actions > 0 {
		spans.push(Span::styled(fmt_action("B", actions.bonus_actions), BONUS_ACTION_COLOR));
		spans.push(Span::raw(DIVIDER_CHARACTER));
	}
	
	if actions.reactions > 0 {
		spans.push(Span::styled(fmt_action("R", actions.reactions), Color::Magenta));
		spans.push(Span::raw(DIVIDER_CHARACTER));
	}
	
	spans.pop(); // remove the trailing divider
	
	Line::from(spans)
}

// 'b: 'a => b outlives a.
fn make_combat_table<'a, 'b: 'a>(tracker_widget: TrackerWidget<'b>) -> Table<'a> {
	use utility_functions::{combatant_row, mix_colors};
	
	let TrackerWidget { tracker, page, draw_labels } = tracker_widget;
	let page = if let Some(page) = page { page } else { &Page::default() };
	
	let page_length = page.get_combatants().len();
	
	let combatants = page
		.get_combatants()
		.iter()
		.map(|i| &tracker.combatants[*i])
		.collect::<Vec<_>>();
	
	let selection = if draw_labels
		&& let Some(select) = page.get_selection()
	{
		**select
	} else {
		LabelSelection::default()
	};
	
	let iter = combatants
		.into_iter()
		.enumerate()
		.map(
			|(index, combatant)| {
				let is_owner_of_turn = index + page.get_id() * page_length == tracker.turn;
				let is_label_selected = draw_labels && selection.label_is_active(index);
				
				let label = if draw_labels {
					LabelSelection::index_to_label(page.get_combatants()[index], page_length)
				} else { None };
				
				let row = combatant_row(label, combatant);
				
				let mut style = Style::default();
				let mut bg_color = None;
				
				if is_label_selected { style = style.bold() }
				
				if combatant.hit_points <= 0 {
					bg_color = bg_color
						.map(|current| mix_colors((255, 0, 0), current))
						.or(Some((100, 0, 0)))
				}
				if is_owner_of_turn {
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
			}
		);
	
	Table::new(
		iter,
		[
			Constraint::Length(2), // label
			Constraint::Fill(2),   // name
			Constraint::Fill(1),   // actions
			Constraint::Fill(1),   // hp / max hp
			Constraint::Fill(1),   // conditions
		]
	)
		.header(Row::new([
			Text::raw(""),
			Text::from("Name").centered(),
			Text::from("Actions").centered(),
			Text::from("HP").centered(),
			Text::from("Conditions").centered(),
		]).bold())
}

mod utility_functions {
	// Imports //
	
	use crate::widgets::{CompactConditions, HitPoints};
	use crate::widgets::tracker::action_line;
	use h5t_core::Combatant;
	use ratatui::prelude::*;
	use ratatui::widgets::*;

	// Functions //
	
	/// Mix two RGB colors together.
	pub(super) fn mix_colors(color1: (u8, u8, u8), color2: (u8, u8, u8)) -> (u8, u8, u8) {
		let f = |n1: u8, n2: u8| -> u8 {
			let n1 = (255 - n1) as f32;
			let n2 = (255 - n2) as f32;
			((n1.powi(2) + n2.powi(2)).sqrt() / 2.0) as u8
		};
		
		(
			f(color1.0, color2.0),
			f(color1.1, color2.1),
			f(color1.2, color2.2),
		)
	}
	
	/// Format multiple actions in a compact way (e.g. `ACT:4 | R`).
	pub(super) fn fmt_action(label: &str, count: u32) -> String {
		if count == 1 { label.repeat(count as usize) }
		else { format!("{}:{}", label, count) }
	}
	
	/// Builds a table [`Row`] for a combatant.
	pub(super) fn combatant_row(label: Option<char>, combatant: &'_ Combatant) -> Row<'_> {
		let label_text = label
			.map(|l| Text::from(format!("{}", l)).bold())
			.unwrap_or_default();
		
		Row::new([
			label_text,
			Text::from(combatant.name()),
			action_line(combatant.actions).centered().into(),
			HitPoints::new(combatant).line().centered().into(),
			CompactConditions::new(combatant).line().into(),
		])
	}
}

// Widget that render's the initiative [`Tracker`](CoreTracker).
// #[derive(Debug)]
// pub struct Tracker<'a> {
//     /// The tracker to display.
//     pub tracker: &'a CoreTracker,
//
//     /// State for label mode.
//     pub label_state: LabelModeState,
// }

// impl<'a> Tracker<'a> {
//     pub fn new(tracker: &'a CoreTracker) -> Self {
//         Self { tracker, label_state: LabelModeState::default() }
//     }
//
//     /// Create a new [`Tracker`] widget with the given labels.
//     pub fn with_labels(tracker: &'a CoreTracker, label: LabelModeState) -> Self {
//         Self { tracker, label_state: label }
//     }
// }

// impl<'a> Widget for Tracker<'a> {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         // draw bordered box for the tracker
//         Block::bordered()
//             .border_type(BorderType::Rounded)
//             .border_style(Style::default().fg(Color::White))
//             .title("Initiative Tracker")
//             .render(area, buf);
//
//         let layout = Layout::vertical([
//             Constraint::Length(2), // round and turn
//             Constraint::Fill(1),
//         ])
//             .horizontal_margin(2)
//             .vertical_margin(1) // avoid the border
//             .spacing(1)
//             .split(area);
//         let [round_and_turn, combatants] = [layout[0], layout[1]];
//
//         let text = vec![
//             Line::styled(format!("Round: {}", self.tracker.round + 1), Modifier::BOLD),
//             Line::styled(
//                 format!("Turn: {}/{}", self.tracker.turn + 1, self.tracker.combatants.len()),
//                 Modifier::BOLD
//             ),
//         ];
//         Paragraph::new(text)
//             .wrap(Wrap { trim: true })
//             .render(round_and_turn, buf);
//
//         Widget::render(combatant_table(&self), combatants, buf);
//     }
// }

// Creates a [`Table`] widget for displaying the combatants in the tracker.
// fn combatant_table<'a>(widget: &'a Tracker) -> Table<'a> {
// 	use utility_functions::{combatant_row, mix_colors};
//
// 	Table::new(
// 		widget.tracker.combatants.iter()
// 			  .enumerate()
// 			  .map(|(i, combatant)| {
// 				  let is_current_turn = i == widget.tracker.turn;
//
// 				  let label = widget.label_state.labels.get_by_right(&i).copied();
// 				  let is_label_selected = widget.label_state.selected.contains(&label.unwrap_or_default());
//
// 				  let row = combatant_row(label, combatant);
// 				  let mut style = Style::default();
// 				  if is_label_selected {
// 					  style = style.bold();
// 				  }
//
// 				  let mut bg_color = None;
// 				  if combatant.hit_points <= 0 {
// 					  bg_color = bg_color
// 						  .map(|current| mix_colors((255, 0, 0), current))
// 						  .or(Some((100, 0, 0)));
// 				  }
// 				  if is_current_turn {
// 					  bg_color = bg_color
// 						  .map(|current| mix_colors((0, 48, 130), current))
// 						  .or(Some((0, 48, 130)));
// 				  }
// 				  if is_label_selected {
// 					  bg_color = bg_color
// 						  .map(|current| mix_colors((128, 85, 0), current))
// 						  .or(Some((128, 85, 0)));
// 				  }
//
// 				  let bg_color = bg_color.map(|bg| Color::Rgb(bg.0, bg.1, bg.2)).unwrap_or(Color::Reset);
// 				  style = style.bg(bg_color);
//
// 				  row.style(style)
// 			  }),
// 		[
// 			Constraint::Length(2), // label mode
// 			Constraint::Fill(2),   // name
// 			Constraint::Fill(1),   // actions
// 			Constraint::Fill(1),   // hp / max hp
// 			Constraint::Fill(1),   // conditions
// 		],
// 	)
// 		.header(Row::new([
// 			Text::raw(""),
// 			Text::from("Name").centered(),
// 			Text::from("Actions").centered(),
// 			Text::from("HP / Max HP").centered(),
// 			Text::from("Conditions").centered(),
// 		]).bold())
// }
