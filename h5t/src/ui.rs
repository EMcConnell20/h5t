// -- Imports -- //

use crate::widgets::{max_combatants_visible, CombatantBlock, StatBlock, TrackerWidget};
use crate::state::{AfterKey, ActionState, ApplyCondition, ApplyDamage};

use h5t_core::{Combatant, CombatantKind, Tracker};

use ratatui::prelude::*;
use crossterm::event::{read, Event, KeyCode, KeyEvent};

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

// -- Page Stuff -- //

#[derive(Clone, Debug, Default)]
pub struct Page {
	id: usize, // Page number
	combatants: Vec<usize>, // Vec of combatant indexes in the tracker.
	label_selection: Option<Box<LabelSelection>>, // Option<Box<_>> to save space.
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct PageConfig {
	page_size: usize,
	current_page: usize,
}

impl Page {
	pub fn get_id(&self) -> usize { self.id }
	
	pub fn get_combatants(&self) -> &Vec<usize> { &self.combatants }
	
	pub fn get_selection(&self) -> Option<&Box<LabelSelection>>{
		self.label_selection.as_ref()
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
	
	fn toggle_index(&mut self, index: usize) {
		if let Some(ref mut select) = self.label_selection {
			debug_assert!(index < select.selection.len());
			select.selection[index] = !select.selection[index];
		} else {
			let mut select = Box::new(LabelSelection::new());
			select.selection[index] = true;
			self.label_selection = Some(select);
		}
	}
	
	/// Takes the page's label selection
	fn take_selection(&mut self) -> Option<Box<LabelSelection>> {
		self.label_selection.take()
	}
	
	fn from_combatants(combatants: &Vec<Combatant>, page_size: usize) -> Vec<Self> {
		if page_size == 0 { return Vec::new() };
		
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
	
	fn from_combatants_and_selection(
		combatants: &Vec<Combatant>,
		selections: Vec<usize>,
		page_size: usize
	) -> Vec<Self> {
		if selections.len() == 0 { return Self::from_combatants(combatants, page_size) }
		if page_size == 0 { return Vec::new() }
		
		let mut pages = Vec::new();
		let mut count = combatants.len();
		let mut selections = selections.into_iter().peekable();
		
		'printer : loop {
			let offset = pages.len() * page_size;
			let space = count.min(page_size);
			
			let mut page = Self {
				id: pages.len(),
				combatants: Vec::with_capacity(space),
				label_selection: None,
			};
			
			(offset..(offset + space)).for_each(|i| page.combatants.push(i));
			
			while let Some(index) = selections.peek() {
				let idx = *index;
				if idx < offset + page_size {
					page.toggle_index(idx);
					selections.next();
				} else { break }
			}
			
			pages.push(page);
			
			if count > page_size { count -= page_size } else { break 'printer }
		}
		
		pages
	}
}

impl PageConfig {
	fn new<B: Backend>(terminal: &Terminal<B>) -> Self {
		Self {
			page_size: max_combatants_visible(terminal.size().unwrap_or_default()),
			current_page: 0,
		}
	}
	
	/// Updates the page configuration.
	///
	/// Rewrites the pages if the configuration was modified.
	fn update<B: Backend>(
		&mut self,
		pages: &mut Vec<Page>,
		terminal: &Terminal<B>,
		tracker: &Tracker,
	) {
		let updated_page_size = max_combatants_visible(terminal.size().unwrap_or_default());
		if self.page_size != updated_page_size {
			let selections = self.take_page_selections(pages);
			
			self.page_size = updated_page_size;
			
			*pages = Page::from_combatants_and_selection(
				&tracker.combatants,
				selections,
				updated_page_size,
			);
			
			if self.current_page >= pages.len() {
				if pages.len() == 0 { self.current_page = 0 }
				else { self.current_page = pages.len() - 1 }
			}
		}
	}
	
	fn take_page_selections(&mut self, pages: &mut Vec<Page>) -> Vec<usize> {
		let mut selections = Vec::new();
		
		let mut iter = 0;
		
		for page in pages {
			if let Some(page_selection) = page.take_selection() {
				for i in 0..self.page_size {
					if page_selection.selection[i] {
						selections.push(iter);
					}
					
					iter += 1;
				}
			} else {
				iter += self.page_size;
			}
		}
		
		
		selections
	}
}

// -- UI Struct -- //

/// A wrapper around a [`Tracker`] that handles UI-dependent logic such as label mode.
#[derive(Debug)]
pub struct Ui<B: Backend> {
    /// The display terminal.
    pub terminal: Terminal<B>,
    /// The initiative tracker.
    pub tracker: Tracker,

	/// Page configuration style
	page_config: PageConfig,
	/// Combatant pages
	pages: Vec<Page>,
	/// Whether label selection mode is enabled
	labels_enabled: bool,
    /// Current info block display mode
	info_block_mode: InfoBlockMode,
	/// (optional) Current action being applied
	action_mode: Option<ActionState>,
	// (optional) Current label mode
    // label_state: Option<LabelModeState>,
}

impl<B: Backend> Ui<B> {
    pub fn new(terminal: Terminal<B>, tracker: Tracker) -> Self {
		let page_config = PageConfig::new(&terminal);
		let pages = Page::from_combatants(&tracker.combatants, page_config.page_size);
		
        Self {
            terminal, tracker,
			page_config, pages,
			labels_enabled: false,
            info_block_mode: InfoBlockMode::CombatState,
            action_mode: None,
            // label_state: None,
        }
    }

    pub fn run(&mut self) {
		'run_loop : loop {
			self.page_config.update(&mut self.pages, &self.terminal, &self.tracker);
			
            self.draw().unwrap();
			
			let key_input = self.get_key_input();

            // Handle any active tracker state.
            if let Some(mut state) = self.action_mode.take() {
                match state.handle_key(key_input) {
                    AfterKey::Exit => state.apply(&mut self.tracker),
                    AfterKey::Stay => self.action_mode = Some(state),
                }
				
                continue 'run_loop;
            }
			
			// Handle regular input.
            match key_input.code {
				KeyCode::Up => // Previous Page
					if self.page_config.current_page > 0 {
						self.page_config.current_page -= 1
					},
				
				KeyCode::Down => // Next Page
					if self.page_config.current_page + 1 < self.pages.len() {
						self.page_config.current_page += 1
					},
				
                KeyCode::Char('c') => {
                    self.action_mode = Some(ActionState::Condition(ApplyCondition::default()));
                },
				
                KeyCode::Char('d') => {
                    let selected = self.enter_label_mode();
                    self.action_mode = Some(ActionState::Damage(ApplyDamage::new(selected)));
                },
				
                KeyCode::Char('a') => { self.tracker.use_action(); }
                KeyCode::Char('b') => { self.tracker.use_bonus_action(); }
                KeyCode::Char('r') => { self.tracker.use_reaction(); }
				
                KeyCode::Char('s') => self.info_block_mode.toggle(),
                KeyCode::Char('n') => self.tracker.next_turn(),
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
			
			let tracker_widget = TrackerWidget::new(
				&self.tracker,
				self.pages.get(self.page_config.current_page),
				self.labels_enabled,
			);
			
			frame.render_widget(tracker_widget, tracker_area);
			
            let combatant = self.tracker.current_combatant();
			
			match self.info_block_mode {
				InfoBlockMode::CombatState =>
					frame.render_widget(CombatantBlock::new(combatant), info_area),
				
				InfoBlockMode::Stats => {
					// TEMP Need to expand this for other combatant kinds
					let CombatantKind::Monster(monster) = &combatant.kind;
					frame.render_widget(StatBlock::new(monster), info_area);
				}
			}
			
            let Some(state) = self.action_mode.as_ref() else { return };
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
		
		self.labels_enabled = true;
		
        'select_loop: loop {
            self.draw().unwrap();
			
			let key_input = self.get_key_input();
			
			match key_input.code {
				KeyCode::Enter => // Confirm Selections
					break 'select_loop,
				
				KeyCode::Esc => // Cancel Selections
					return Vec::new(),
				
				KeyCode::Up => // Previous Page
					if self.page_config.current_page > 0 {
						self.page_config.current_page -= 1
					},
				
				KeyCode::Down => // Next Page
					if self.page_config.current_page + 1 < self.pages.len() {
						self.page_config.current_page += 1
					},
				
				KeyCode::Char(label) =>
					self.pages[self.page_config.current_page].toggle_selection(label),
				
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
					final_selection.push(i + page.id * self.page_config.page_size)
				}
			}
		}
		
		final_selection
    }
	
	fn get_key_input(&mut self) -> KeyEvent {
		'get_key_input: loop {
			let Ok(event) = read() else { continue 'get_key_input };
			match event {
				Event::Key(key) => break 'get_key_input key,
				
				Event::Resize(_, _) => {
					self.page_config.update(&mut self.pages, &self.terminal, &self.tracker);
					self.draw().unwrap();
				}
				
				_ => (),
			}
		}
	}
}

impl<B: Backend> Widget for Ui<B> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		TrackerWidget::new(
			&self.tracker,
			self.pages.get(self.page_config.current_page),
			self.labels_enabled,
		).render(area, buf);
	}
}

impl<B: Backend> Drop for Ui<B> {
	fn drop(&mut self) { ratatui::restore() }
}

// NOTE `tracker` is already a public field, so these implementations aren't necessary.
// impl<B: Backend> Deref for UI<B> {
//     type Target = Tracker;
//
//     fn deref(&self) -> &Self::Target { &self.tracker }
// }
//
// impl<B: Backend> DerefMut for UI<B> {
//     fn deref_mut(&mut self) -> &mut Self::Target { &mut self.tracker }
// }
