use bevy::prelude::*;

use crate::GameState;

pub struct WinPlugin;

impl Plugin for WinPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(Win::handle);
	}
}

#[derive(Event)]
struct Win;

impl Win {
	fn handle(_: On<Self>, mut game_state: ResMut<NextState<GameState>>) {
		game_state.set(GameState::WinGame);
	}
}
