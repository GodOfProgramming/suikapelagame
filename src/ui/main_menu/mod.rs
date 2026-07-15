use crate::GameState;
use bevy::prelude::*;
use bevy_svg::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GameState::MainMenu), title.spawn())
			.add_systems(OnExit(GameState::MainMenu), on_exit);
	}
}

#[derive(SceneComponent, Default, Clone)]
#[require(Transform, Visibility)]
pub struct MainMenu;

impl MainMenu {
	fn scene() -> impl Scene {
		bsn! {
			Children [
				title()
			]
		}
	}
}

fn on_exit(mut commands: Commands, q_main_menu: Query<Entity, With<MainMenu>>) {
	for entity in q_main_menu {
		commands.entity(entity).despawn();
	}
}

fn title() -> impl Scene {
	bsn! {
		Name::new("Title Text")
		Svg2d("title.svg")
		Origin::Custom({(0.5, 3.6)})
	}
}
