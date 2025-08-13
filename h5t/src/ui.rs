// -- Imports -- //

use crate::widgets::Tracker as TrackerWidget;
use crate::widgets::{max_combatants_visible, CombatantBlock, StatBlock};
use crate::state::{AfterKey, ActionState, ApplyCondition, ApplyDamage};

use h5t_core::{Combatant, CombatantKind, Tracker};

use ratatui::prelude::*;
use crossterm::event::{read, Event, KeyCode};
use bimap::BiMap;

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

// -- Label Selection -- //

/// Labels used for label mode. The tracker will choose labels from this string in sequential
/// order.
///
/// The sequence of labels is simply the characters on a QWERTY keyboard going column by column.
/// This keeps labels physically close to each other on the keyboard.
pub(crate) const LABELS: &str = "qazwsxedcrfvtgbyhnujmik,ol.p;/[']";

/// The label selection state of the tracker.
#[derive(Copy, Clone, Debug, Default)]
pub struct LabelSelection {
	/// None if not in selection mode. <br>
	/// Some if labels are being selected.
	selection: [bool; 32],
}

impl LabelSelection {
	pub const fn new() -> Self { Self { selection: [false; 32] } }
	
	pub const fn label_is_active(&self, index: usize) -> bool {
		debug_assert!(index < 32);
		self.selection[index]
	}
	
	pub fn select(&mut self, label: char, label_count: usize) {
		let Some(index) = Self::label_to_index(label, label_count) else { return };
		debug_assert!(index < 32);
		self.selection[index] = !self.selection[index];
	}
	
	pub fn wipe_selection(&mut self) {
		self.selection = [false; 32]
	}
	
	/// Converts a label to an index if the label is on screen.
	///
	/// label - The label character. <br>
	/// label_count - The number of labels being displayed.
	pub const fn label_to_index(label: char, label_count: usize) -> Option<usize> {
		let index = match label {
			'q' => 0,
			'a' => 1,
			'z' => 2,
			'w' => 3,
			's' => 4,
			'x' => 5,
			'e' => 6,
			'd' => 7,
			'c' => 8,
			'r' => 9,
			'f' => 10,
			'v' => 11,
			't' => 12,
			'g' => 13,
			'b' => 14,
			'y' => 15,
			'h' => 16,
			'n' => 17,
			'u' => 18,
			'j' => 19,
			'm' => 20,
			'i' => 21,
			'k' => 22,
			',' => 23,
			'l' => 24,
			'.' => 25,
			'p' => 26,
			';' => 27,
			'/' => 28,
			'[' => 29,
			'\'' => 30,
			']' => 31,
			_ => return None,
		};
		
		// This ensures that only the labels shown on screen are selectable.
		if index < label_count { Some(index) } else { None }
	}
	
	/// Converts an index to a label if the label is on screen.
	///
	/// index - The label index. <br>
	/// label_count - The number of labels being displayed.
	pub const fn index_to_label(index: usize, label_count: usize) -> Option<char> {
		let label = match index {
			0 => 'q',
			1 => 'a',
			2 => 'z',
			3 => 'w',
			4 => 's',
			5 => 'x',
			6 => 'e',
			7 => 'd',
			8 => 'c',
			9 => 'r',
			10 => 'f',
			11 => 'v',
			12 => 't',
			13 => 'g',
			14 => 'b',
			15 => 'y',
			16 => 'h',
			17 => 'n',
			18 => 'u',
			19 => 'j',
			20 => 'm',
			21 => 'i',
			22 => 'k',
			23 => ',',
			24 => 'l',
			25 => '.',
			26 => 'p',
			27 => ';',
			28 => '/',
			29 => '[',
			30 => '\'',
			31 => ']',
			_ => return None,
		};
		
		// This ensures that only the labels shown on screen are selectable.
		if index < label_count { Some(label) } else { None }
	}
}

// -- Old Label Select -- //

/// State passed to [`TrackerWidget`] to handle label mode.
#[derive(Clone, Debug, Default)]
pub struct LabelModeState {
	/// The labels to display next to each combatant.
	pub labels: BiMap<char, usize>,
	
	/// The labels that have been selected.
	pub selected: HashSet<char>,
}

impl LabelModeState {
	fn new(labels: BiMap<char, usize>, selected: HashSet<char>) -> Self {
		Self { labels, selected }
	}
}

// -- Info Block -- //

/// The type of info being displayed in the UI info block.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InfoBlockMode {
    /// Combatant's combat state.
	CombatState,
    /// Combatant's primary stats (mostly useful for monsters).
	Stats,
}

impl InfoBlockMode {
    /// Cycle info block mode.
    pub fn toggle(&mut self) {
        *self = match self {
            InfoBlockMode::Stats => InfoBlockMode::CombatState,
            InfoBlockMode::CombatState => InfoBlockMode::Stats,
        };
    }
}

// -- Page Struct -- //

#[derive(Clone, Debug)]
pub struct Page {
	id: usize, // Page number
	combatants: Vec<usize>, // Vec of combatant indexes in the tracker.
	label_selection: Option<Box<LabelSelection>>, // Option<Box<_>> to save space.
}

impl Page {
	pub fn get_id(&self) -> usize { self.id }
	
	pub fn get_combatants(&self) -> &Vec<usize> { &self.combatants }
	
	pub fn get_selection(&self) -> Option<&Box<LabelSelection>>{
		self.label_selection.as_ref()
	}
	
	fn from_combatants(combatants: &Vec<Combatant>, page_size: usize) -> Vec<Self> {
		let mut pages = Vec::new();
		let mut count = combatants.len();
		
		'printer : loop {
			let offset = pages.len() * page_size;
			let space = count.min(page_size);
			
			let mut page = Self {
				id: pages.len(),
				combatants: Vec::with_capacity(space),
				label_selection: None,
			};
			
			(offset..(offset + space)).for_each(|i| page.combatants.push(i));
			
			pages.push(page);
			if count > page_size { count -= page_size }
			else { break 'printer }
		}
		
		pages
	}
	
	fn toggle_selection(&mut self, label: char) {
		if let Some(ref mut select) = self.label_selection {
			select.select(label, self.combatants.len());
		} else {
			let mut select = Box::new(LabelSelection::new());
			select.select(label, self.combatants.len());
			self.label_selection = Some(select);
		}
	}
	
	/// Takes the page's label selection
	fn take_selection(&mut self) -> Option<Box<LabelSelection>> {
		self.label_selection.take()
	}
}

// -- UI Struct -- //

/// A wrapper around a [`Tracker`] that handles UI-dependent logic such as label mode.
#[derive(Debug)]
pub struct UI<B: Backend> {
    /// The display terminal.
    pub terminal: Terminal<B>,
    /// The initiative tracker.
    pub tracker: Tracker,

	/// Combatant pages
	pages: Vec<Page>,
	current_page: usize,
	
	/// Whether label selection mode is enabled
	labels_enabled: bool,
    /// Current info block display mode
	info_block_mode: InfoBlockMode,
	/// (optional) Current action being applied
	action_state: Option<ActionState>,
	/// (optional) Current label mode
    label_state: Option<LabelModeState>,
}

impl<B: Backend> UI<B> {
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
		let pages = Page::from_combatants(
			&tracker.combatants,
			max_combatants_visible(terminal.size().unwrap())
		);
		
        Self {
            terminal,
            tracker,
			pages,
			current_page: 0,
			labels_enabled: false,
            info_block_mode: InfoBlockMode::CombatState,
			
            action_state: None,
            label_state: None,
        }
    }

    pub fn run(&mut self) {
		'run_loop : loop {
            self.draw().unwrap();
			
			let key_input = 'get_key_input: loop {
				if let Ok(Event::Key(key)) = read() {
					break 'get_key_input key;
				}
			};

            // Handle any active tracker state.
            if let Some(mut state) = self.action_state.take() {
                match state.handle_key(key_input) {
                    AfterKey::Exit => state.apply(&mut self.tracker),
                    AfterKey::Stay => self.action_state = Some(state),
                }
				
                continue 'run_loop;
            }
			
			// Handle regular input.
            match key_input.code {
                KeyCode::Char('c') => {
                    self.action_state = Some(ActionState::Condition(ApplyCondition::default()));
                },
				
                KeyCode::Char('d') => {
                    let selected = self.enter_label_mode();
                    self.action_state = Some(ActionState::Damage(ApplyDamage::new(selected)));
                    self.label_state = None;
                },
				
                KeyCode::Char('a') => { self.use_action(); }
                KeyCode::Char('b') => { self.use_bonus_action(); }
                KeyCode::Char('r') => { self.use_reaction(); }
				
                KeyCode::Char('s') => self.info_block_mode.toggle(),
                KeyCode::Char('n') => self.next_turn(),
                KeyCode::Char('q') => break 'run_loop,
				
                _ => (),
            }
        }
    }

    pub fn draw(&'_ mut self) -> std::io::Result<ratatui::CompletedFrame<'_>> {
        self.terminal.draw(|frame| {
            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]).split(frame.area());
            let [tracker_area, info_area] = [layout[0], layout[1]];
			
			
			
            // show tracker
            let tracker_widget = if let Some(label) = &self.label_state {
                TrackerWidget::with_labels(&self.tracker, label.clone())
            } else {
                TrackerWidget::new(&self.tracker)
            };
            frame.render_widget(tracker_widget, tracker_area);

            let combatant = self.tracker.current_combatant();
            if self.info_block_mode == InfoBlockMode::Stats {
                // show stat block in place of the combatant card
                let CombatantKind::Monster(monster) = &combatant.kind;
                frame.render_widget(StatBlock::new(monster), info_area);
            } else {
                // show combatant card
                frame.render_widget(CombatantBlock::new(combatant), info_area);
            }

            let Some(state) = self.action_state.as_ref() else {
                return;
            };
            state.draw(frame);
        })
    }

    /// Enters label mode.
    ///
    /// Label mode is a special state where the user can quickly select one or more combatants
    /// to apply an action to. This works by displaying a label next to each combatant's name, and
    /// the user can press the corresponding key to toggle the label on or off.
    ///
    /// This function blocks until the user selects the combatants and presses the `Enter` key,
    /// returning mutable references to the selected combatants.
    pub fn enter_label_mode(&mut self) -> Vec<usize> {
		// If there aren't pages, no selections can be made.
		if self.pages.len() == 0 { return Vec::new() }
		
		// TODO Remove this
        // let size = self.terminal.size().unwrap();
        // let combatants = max_combatants_visible(size).min(self.tracker.combatants.len());

		self.labels_enabled = true;
		
		// TODO Remove this
        // generate labels for all combatants in view
        // let combatant_label_map = (0..combatants)
        //     // .skip(self.turn) // TODO: change when pagination is implemented
        //     .map(|i| (LABELS.chars().nth(i).unwrap(), i))
        //     .collect::<BiMap<_, _>>();

		// TODO Remove this
        // let mut selected_labels = HashSet::new();
		
        'select_loop: loop {
			// TODO Remove this
			// self.label_state = Some(
			// 	LabelModeState::new(combatant_label_map.clone(), selected_labels.clone())
			// );
			
            self.draw().unwrap();
			
			let key_input = 'get_key_input: loop {
				if let Ok(Event::Key(key)) = read() {
					break 'get_key_input key;
				}
			};
			
			match key_input.code {
				KeyCode::Enter => // Confirm Selections
					break 'select_loop,
				
				KeyCode::Esc => // Cancel Selections
					return Vec::new(),
				
				KeyCode::Up => // Previous Page
					if self.current_page + 1 < self.pages.len() { self.current_page += 1 },
				
				KeyCode::Down => // Next Page
					if self.current_page > 0 { self.current_page -= 1 },
				
				KeyCode::Char(label) => {
					self.pages[self.current_page].toggle_selection(label);
					
					// TODO Remove this
					// if combatant_label_map.contains_left(&label) {
					// 	if selected_labels.contains(&label) {
					// 		selected_labels.remove(&label);
					// 	} else {
					// 		selected_labels.insert(label);
					// 	}
					// }
				},
				
				_ => (),
			}
        }
		
		self.labels_enabled = false;
		
		// Collect selections from pages.
		let mut final_selection = Vec::new();
		for page in &mut self.pages {
			let Some(selections) = page.take_selection() else { continue };
			
			for i in 0..page.combatants.len() {
				if selections.selection[i] {
					final_selection.push(i + page.id)
				}
			}
		}

		// TODO Remove this
        // return selected combatants
        // selected_labels
        //     .into_iter()
        //     .filter_map(|label| combatant_label_map.get_by_left(&label).copied())
        //     .collect()
		
		final_selection
    }
}

impl<B: Backend> Widget for UI<B> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		TrackerWidget::new(&self.tracker).render(area, buf);
	}
}

impl<B: Backend> Drop for UI<B> {
	fn drop(&mut self) { ratatui::restore() }
}

// NOTE `tracker` is already a public field, so these implementations aren't necessary.
impl<B: Backend> Deref for UI<B> {
    type Target = Tracker;

    fn deref(&self) -> &Self::Target { &self.tracker }
}

impl<B: Backend> DerefMut for UI<B> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.tracker }
}
