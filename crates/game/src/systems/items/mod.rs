mod commands;
mod merge;
mod motion;
mod pickup;
mod player_drop;
mod spawn;

pub use commands::apply_inventory_commands_system;
pub use merge::item_merge_system;
pub use motion::item_motion_system;
pub use pickup::{drop_position_in_front, item_pickup_system, item_within_pickup_reach};
pub use player_drop::player_drop_items_system;
pub use spawn::spawn_drops_on_block_break;

use engine_core::{App, Stage, SystemId};

pub fn register_authoritative_item_systems(app: &mut App, after_mining: SystemId) {
    app.add_system_after(Stage::Update, spawn_drops_on_block_break, after_mining);
    app.add_system(Stage::Update, player_drop_items_system);
    app.add_system(Stage::Update, apply_inventory_commands_system);
    let motion = app.add_system(Stage::Physics, item_motion_system);
    let merge = app.add_system_after(Stage::Physics, item_merge_system, motion);
    app.add_system_after(Stage::Physics, item_pickup_system, merge);
}
