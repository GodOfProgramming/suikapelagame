pub mod components;
pub mod main_menu;

use crate::{GameCamera, ui::main_menu::MainMenuPlugin};
use bevy::prelude::*;
use components::{FruitCircleComponent, LoginFormComponent, NextFruitComponent, ScoreComponent};

#[macro_export]
macro_rules! component_name {
	($name:ident) => {{
		let _: Option<$name> = None;
		bevy::scene::template_value(bevy::ecs::name::Name::new(stringify!($name)))
	}};
}

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
				display: Display::Flex,
				flex_direction: FlexDirection::Row,
				width: percent(100),
				height: percent(100),
				justify_content: JustifyContent::SpaceBetween,
				align_items: AlignItems::Stretch,
			}
			Children [
				left(),
				right(),
			]
			on(|event: On<Add, Node>, mut commands: Commands, game_camera: Single<Entity, With<GameCamera>>| {
				commands.entity(event.event_target()).insert(UiTargetCamera(*game_camera));
			})
	}
}

fn left() -> impl Scene {
	bsn! {
		Name::new("Left")
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			width: percent(20),
			height: percent(100),
			justify_content: JustifyContent::SpaceBetween,
			align_items: AlignItems::Stretch,
		}
		Children [
			@ScoreComponent Node {
				margin: { UiRect::left(px(48)).with_top(px(176)) }
			},
			@LoginFormComponent Node {
				margin: { UiRect::left(px(48)).with_bottom(px(160)) }
			},
		]
	}
}

fn right() -> impl Scene {
	bsn! {
		Name::new("Right")
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			width: percent(20),
			height: percent(100),
			justify_content: JustifyContent::SpaceBetween,
			align_items: AlignItems::Stretch,
		}
		Children [
			@NextFruitComponent Node {
				width: percent(100),
				margin: { UiRect::left(px(-48)).with_top(px(176)) },
			},
			@FruitCircleComponent Node {
				width: percent(100),
				margin: { UiRect::left(px(-48)).with_bottom(px(128)) }
			},
		]
	}
}
