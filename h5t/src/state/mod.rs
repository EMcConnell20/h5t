// -- Modules -- //

pub mod apply_condition;
pub mod apply_damage;

// -- Imports -- //

use h5t_core::Tracker;

// -- Exports -- //

pub use apply_damage::ApplyDamage;
pub use apply_condition::ApplyCondition;

/// What to do after handling a key event.
#[derive(Default)]
pub enum AfterKey {
    /// Stay in the current state.
    #[default]
    Stay,
    /// Exit and hand control back to the main loop.
    Exit,
}

/// State of an action being applied through the [`Tracker`].
///
/// `::Condition()` Applying a condition. <br>
/// `::Damage()` Applying damage.
#[derive(Debug, Clone)]
pub enum ActionState {
    /// Applying a condition to combatant(s).
	Condition(ApplyCondition),
    /// Applying damage to combatant(s).
	Damage(ApplyDamage),
}

impl ActionState {
	// TODO Move to Drawable trait
    /// Allow the state to draw itself.
    pub fn draw(&self, frame: &mut ratatui::Frame) {
        match self {
            Self::Condition(state) => state.draw(frame),
            Self::Damage(state) => state.draw(frame),
        }
    }

	// TODO Move to InputHandler trait
    /// Handle a key event.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> AfterKey {
        match self {
            Self::Condition(state) => state.handle_key(key),
            Self::Damage(state) => state.handle_key(key),
        }
    }

    /// Apply the action to the tracker. This function is called when the state is exited.
    pub fn apply(self, tracker: &mut Tracker) {
        match self {
            Self::Condition(state) => state.apply(tracker),
            Self::Damage(state) => state.apply(tracker),
        }
    }
}
