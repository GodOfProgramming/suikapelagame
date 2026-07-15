mod debug;
mod game;
mod ui;

use crate::{debug::DebugPlugin, ui::UiPlugin};
use beditor::prelude::*;
use bevy::prelude::*;
use bevy_defer::AsyncPlugin;
use bevy_svg::SvgPlugin;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
	let mut app = App::new();

	if cfg!(debug_assertions) {
		app
			.add_plugins(EditorPlugin::new().register_game_camera::<GameCamera>())
			.add_plugins(DebugPlugin);
	}

	app
		.add_plugins((AsyncPlugin::default_settings(), SvgPlugin))
		.insert_resource(ClearColor(Color::srgb_u8(0xff, 0xd5, 0x9d)))
		.insert_state(GameState::Startup)
		.add_systems(Startup, (static_entities.spawn(), init))
		.add_systems(OnEnter(GameState::InGame), game_ui.spawn())
		.add_plugins(UiPlugin)
		.run();
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
	Startup,
	MainMenu,
	InGame,
}

#[derive(Component, Reflect, Default, Identifiable, Clone)]
#[id("ea4a1225-0fda-4e8d-bd49-7eb895372c8a")]
#[require(Camera2d, Name = Name::new("Game Camera"))]
struct GameCamera;

fn static_entities() -> impl SceneList {
	bsn_list! {
		GameCamera
	}
}

fn init(mut next_state: ResMut<NextState<GameState>>) {
	next_state.set(GameState::MainMenu);
}

fn game_ui() -> impl Scene {
	bsn! {}
}
