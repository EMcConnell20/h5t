// -- Modules -- //

mod selectable;
mod state;
mod ui;
mod widgets;

// -- Imports -- //

use ui::UI;
use h5t_core::{CombatantKind, Monster, Tracker};

// -- Main -- //

fn main() {
    // NOTE: monster JSON data provided courtesy of https://www.dnd5eapi.co/
    let file = std::fs::File::open("data/monsters.json").unwrap();
    let monsters = serde_json::from_reader::<_, Vec<Monster>>(file).unwrap();
    // println!("{:#?}", monsters);

    let mut tracker = UI::new(
        ratatui::init(),
        Tracker::new(monsters
            .into_iter()
            .map(|m| CombatantKind::Monster(m).into())
            .collect::<Vec<_>>()),
    );

    tracker.run();
}
