use crate::{GameState, component_name, ui::components::FruitImage};
use bevy::prelude::*;
use bevy_svg::prelude::*;
use rand::Rng;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnExit(GameState::MainMenu), on_exit)
			.add_systems(Update, rotate_fruits);
	}
}

#[derive(SceneComponent, Default, Clone)]
#[require(Transform, Visibility)]
pub struct MainMenuComponent;

impl MainMenuComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(MainMenuComponent) }
			Node {
				width: percent(100),
				height: percent(100),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
			}
			Children [
				@TitleComponent,
				@BlenderComponent Node {
					margin: { UiRect::top(px(128)) }
				},
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone, Reflect)]
struct TitleComponent;

impl TitleComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(TitleComponent) }
			Svg2d("title.svg")
			Origin::Custom({(0.5, 3.6)})
		}
	}
}

#[derive(SceneComponent, Default, Clone, Reflect)]
struct BlenderComponent;

impl BlenderComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(BlenderComponent) }
			Node {
				width: percent(60),
				aspect_ratio: {Some(1.0)}
			}
			ImageNode {
				image: "binary/blender.png"
			}
			Children [
				@SpinningFruitComponent
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone, Reflect)]
struct SpinningFruitComponent;

#[derive(Component, Clone)]
struct SpinningFruit {
	speed: f32,
	clockwise: bool,
}

impl Default for SpinningFruit {
	fn default() -> Self {
		Self {
			speed: rand::random_range(0.5..1.0),
			clockwise: { rand::rng().next_u32() & 1 == 0 },
		}
	}
}

impl SpinningFruitComponent {
	fn scene() -> impl Scene {
		const WIDTH: f32 = 40.0;

		bsn! {
			{ component_name!(SpinningFruitComponent) }
			Node {
				width: percent(100),
				position_type: PositionType::Relative,
				aspect_ratio: { Some(1.0) }
			}
			Children [
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/strawberry.png",
					@degrees: 90.0,
				},
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/melon.png",
					@degrees: 30.0,
				},
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/peach.png",
					@degrees: 330.0,
				},
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/persimmon.png",
					@degrees: 270.0,
				},
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/grape.png",
					@degrees: 210.0,
				},
				SpinningFruit
				@FruitImage {
					@width: WIDTH,
					@image: "binary/fruit/pear.png",
					@degrees: 150.0,
				},
			]
		}
	}
}

fn on_exit(mut commands: Commands, q_main_menu: Query<Entity, With<MainMenuComponent>>) {
	for entity in q_main_menu {
		commands.entity(entity).despawn();
	}
}

fn rotate_fruits(q_fruits: Query<(&mut UiTransform, &SpinningFruit)>, time: Res<Time>) {
	for (mut tf, spin) in q_fruits {
		let rads = time.delta_secs() * spin.speed * if spin.clockwise { 1.0 } else { -1.0 };
		let rotation = Rot2::radians(rads);
		tf.rotation *= rotation;
	}
}
