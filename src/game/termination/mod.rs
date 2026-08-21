pub mod lose;
pub mod win;

use bevy::prelude::*;
use lose::LosePlugin;
use win::WinPlugin;

pub struct TerminationPlugin;

impl Plugin for TerminationPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((WinPlugin, LosePlugin));
	}
}
