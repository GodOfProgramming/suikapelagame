pub mod components;
pub mod main_menu;

use crate::{GameCamera, GameState, ui::main_menu::MainMenuPlugin};
use bevy::{
	math::{I16Vec2, i16vec2},
	prelude::*,
};
use components::{FruitCircleComponent, LoginFormComponent, NextFruitComponent, ScoreComponent};
use main_menu::MainMenu;

const SCORE_GRID_ZONE: I16Vec2 = i16vec2(1, 1);
const LOGIN_GRID_ZONE: I16Vec2 = i16vec2(1, 2);
const NEXT_GRID_ZONE: I16Vec2 = i16vec2(3, 1);
const CIRCLE_GRID_ZONE: I16Vec2 = i16vec2(3, 2);

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(MainMenuPlugin)
			.add_systems(Startup, template.spawn());
	}
}

fn template() -> impl Scene {
	bsn! {
			Name::new("Ui Root")
			Node {
				display: Display::Grid,
				width: percent(100),
				height: percent(100),
				grid_template_rows: vec![GridTrack::vh(33.0), GridTrack::vh(67.0)],
				grid_template_columns: vec![
					GridTrack::fr(1.0),
					GridTrack::max_content(),
					GridTrack::fr(1.0),
				],
			}
			Children [
				@ScoreComponent Node {
					grid_column: GridPlacement::start(SCORE_GRID_ZONE.x),
					grid_row: GridPlacement::start(SCORE_GRID_ZONE.y),
				},
				@LoginFormComponent Node {
					grid_column: GridPlacement::start(LOGIN_GRID_ZONE.x),
					grid_row: GridPlacement::start(LOGIN_GRID_ZONE.y),
					margin: { UiRect::bottom(auto()).with_left(px(16)) }
				},
				@MainMenu,
				@NextFruitComponent Node {
					grid_column: GridPlacement::start(NEXT_GRID_ZONE.x),
					grid_row: GridPlacement::start(NEXT_GRID_ZONE.y),
				},
				@FruitCircleComponent Node {
					grid_column: GridPlacement::start(CIRCLE_GRID_ZONE.x),
					grid_row: GridPlacement::start(CIRCLE_GRID_ZONE.y),
				}
			]
			on(|event: On<Add, Node>, mut commands: Commands, game_camera: Single<Entity, With<GameCamera>>| {
				commands.entity(event.event_target()).insert(UiTargetCamera(*game_camera));
			})
	}
}

fn left() -> impl Scene {
	bsn! {}
}

fn right() -> impl Scene {
	bsn! {}
}
