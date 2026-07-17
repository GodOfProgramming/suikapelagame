mod game;
mod ui;
mod util;

#[cfg(feature = "dev")]
mod debug;

use crate::{
	game::{GAME_RENDER_LAYER, GamePlugin},
	ui::UiPlugin,
};
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use bevy_defer::AsyncPlugin;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_svg::SvgPlugin;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
	let mut app = App::new();

	#[cfg(feature = "dev")]
	{
		use beditor::prelude::*;
		app
			.add_plugins(
				EditorPlugin::new()
					.register_game_camera::<GameCamera>()
					.configure_defaults(|_, defaults| defaults.set(ImagePlugin::default_nearest())),
			)
			.add_plugins(debug::DebugPlugin);
	}
	#[cfg(not(feature = "dev"))]
	{
		app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
	}

	app
		.add_plugins((
			AsyncPlugin::default_settings(),
			PhysicsPlugins::default(),
			EntropyPlugin::<WyRand>::default(),
			SvgPlugin,
		))
		.insert_resource(ClearColor(Color::srgb_u8(0xff, 0xd5, 0x9d)))
		.insert_state(GameState::MainMenu)
		.add_systems(Startup, static_entities.spawn())
		.add_plugins((UiPlugin, GamePlugin))
		.run();
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
	MainMenu,
	InitGameplay,
	NewGame,
	WinGame,
	LoseGame,
}

#[derive(Component, Reflect, Default, Clone)]
#[cfg_attr(
	feature = "dev",
	derive(beditor::prelude::Identifiable),
	id("ea4a1225-0fda-4e8d-bd49-7eb895372c8a")
)]
#[require(Camera2d, Name = Name::new("Game Camera"))]
struct GameCamera;

fn static_entities() -> impl SceneList {
	bsn_list! {
		GameCamera
		template_value(GAME_RENDER_LAYER)
	}
}
