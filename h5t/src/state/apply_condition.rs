// -- Imports -- //

use crate::state::AfterKey;
use crate::selectable::Selectable;
use crate::ui::{LABELS, LabelSelection, Page};
use crate::widgets::popup::{popup_area, Multiselect, Select};

use h5t_core::{Condition, ConditionDuration, ConditionKind};

use ratatui::prelude::*;
use ratatui::layout::Flex;
use crossterm::event::{KeyCode, KeyEvent};

use std::num::NonZeroU32;
use std::collections::{HashMap, HashSet};

// -- Field Info -- //

/// Helper enum to indicate which form field is currently selected.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Field {
    #[default]
    Conditions,
    Duration,
}

/// Helper enum to render condition durations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Unit {
    #[default]
	/// Until the end of next turn
	Turn,
    Round,
    Minute,
    Forever,
}

impl Selectable for Unit {
    const N: usize = 4;

    fn variants() -> impl Iterator<Item = Self> {
        [
            Unit::Turn,
            Unit::Round,
            Unit::Minute,
            Unit::Forever,
        ].into_iter()
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Turn => write!(f, "Until end of next turn"),
            Unit::Round => write!(f, "Rounds"),
            Unit::Minute => write!(f, "Minutes"),
            Unit::Forever => write!(f, "Forever"),
        }
    }
}

// -- Apply Condition -- //

/// State for applying conditions to combatants.
#[derive(Clone, Debug, Default)]
pub struct ApplyCondition {
    /// The conditions to apply to combatants.
    conditions: HashSet<ConditionKind>,

    /// Indicates which form field is currently selected.
    selected: Field,

    /// Value of the input field for the duration.
    value: String,

    /// Duration of the conditions.
    unit: Unit,
}

impl ApplyCondition {
    /// Draw the state to the given [`Frame`].
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let area = popup_area(area, Flex::Center, Flex::End, (area.width, area.height / 3), 0);
        let [conditions, duration] = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .flex(Flex::Center)
            .areas(area);
        frame.render_widget(Multiselect::new(
            "Select condition(s)",
            &self.conditions,
            self.selected == Field::Conditions,
        ), conditions);
        frame.render_widget(Select::new(
            "For how long?",
            &self.unit,
            self.selected == Field::Duration,
        ), duration);
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) -> AfterKey {
        // generate labels for all conditions
        if self.selected == Field::Conditions {
            let label_to_option = LABELS
                .chars()
                .zip(ConditionKind::variants())
                .collect::<HashMap<_, _>>();

            match key.code {
                KeyCode::Esc => return AfterKey::Exit,
                KeyCode::Enter => {
                    self.selected = Field::Duration;
                    return AfterKey::Stay;
                },
                KeyCode::Char(label) => {
                    let selected = &mut self.conditions;
                    if let Some(option) = label_to_option.get(&label) {
                        if selected.contains(option) {
                            selected.remove(option);
                        } else {
                            selected.insert(*option);
                        }
                    }
                },
                _ => (),
            }
        } else {
            let label_to_option = LABELS
                .chars()
                .zip(Unit::variants())
                .collect::<HashMap<_, _>>();

            match key.code {
                KeyCode::Esc => {
                    self.selected = Field::Conditions;
                    return AfterKey::Stay;
                },
                KeyCode::Enter => return AfterKey::Exit,
                KeyCode::Char(label) => {
                    let selected = &mut self.unit;
                    if let Some(option) = label_to_option.get(&label) {
                        *selected = *option;
                    }
                },
                _ => (),
            }
        }

        AfterKey::Stay
    }

    /// Apply the conditions to the tracker.
    pub fn apply(&self, tracker: &mut h5t_core::Tracker) {
        for condition in &self.conditions {
            // TODO: get the duration from the input field
            let duration = match self.unit {
                Unit::Turn => ConditionDuration::UntilNextTurn,
                Unit::Round => ConditionDuration::Rounds(NonZeroU32::new(1).unwrap()),
                Unit::Minute => ConditionDuration::Minutes(NonZeroU32::new(1).unwrap()),
                Unit::Forever => ConditionDuration::Forever,
            };

            // if the condition is already present, override its length if the new one is longer
            // otherwise, add the condition
            let existing_condition = tracker
                .current_combatant_mut()
                .conditions
                .iter_mut()
                .find(|c| c.kind == *condition);

            if let Some(existing_condition) = existing_condition {
                // override its length if the new one is longer
                if duration > existing_condition.duration {
                    existing_condition.duration = duration;
                }
            } else {
                // add new condition
                tracker.current_combatant_mut().conditions.push(Condition {
                    kind: *condition,
                    duration,
                });
            }
        }
    }
}
