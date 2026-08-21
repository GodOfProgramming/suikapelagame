use crate::{
	GameState, component_name,
	ui::components::{ClickableButton, FruitImage},
};
use bevy::{
	camera::{ImageRenderTarget, RenderTarget, visibility::RenderLayers},
	color::palettes::tailwind::ORANGE_500,
	prelude::*,
	render::render_resource::TextureFormat,
};
use bevy_svg::prelude::*;
use rand::Rng;

const TITLE_RENDER_LAYER: RenderLayers = RenderLayers::layer(2);

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, rotate_fruits);
	}
}

#[derive(SceneComponent, Default, Clone)]
#[require(Transform, Visibility)]
pub struct StartMenuComponent;

impl StartMenuComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(StartMenuComponent) }
			Transform Visibility
			Node {
				width: percent(100),
				height: percent(100),
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
			}
			Children [
				@TitleComponent,
				@BlenderComponent,
				@StartButtonComponent,
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
			Node {
				width: percent(100),
			}
			Transform Visibility
			on(|event: On<Add, Self>, mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>| {
				let image = Image::new_target_texture(1200, 200, TextureFormat::Rgba8UnormSrgb, None);
				let image_handle = images.add(image);

				commands.entity(event.event_target())
				.insert(ImageNode::new(image_handle.clone()))
				.with_children(|commands| {
					commands.spawn((
						Camera2d,
						TITLE_RENDER_LAYER,
						RenderTarget::Image(ImageRenderTarget {
							handle: image_handle,
							scale_factor: 1.0,
						})
					));

					commands.spawn((
						Svg2d(asset_server.load("title.svg")),
						Origin::Center,
						TITLE_RENDER_LAYER
					));
				});
			})
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
				height: px(0),
				flex_grow: 1.0,
				width: percent(60),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				aspect_ratio: {Some(1.0)}
			}
			Children [
				ImageNode {
					image: "binary/blender.png"
				} Node {
					position_type: PositionType::Absolute,
					width: percent(100),
					height: percent(100),
				},
				@SpinningFruitComponent Node {
					height: percent(100),
				},
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

#[derive(SceneComponent, Default, Clone)]
struct StartButtonComponent;

impl StartButtonComponent {
	fn scene() -> impl Scene {
		bsn! {
			@ClickableButton
			BackgroundColor(ORANGE_500)
			Node {
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				border_radius: BorderRadius::all(px(16)),
				aspect_ratio: {Some(6.0 / 1.0)},
				width: px(300),
			}
			Children [
				Text("Start") TextFont { weight: FontWeight(900), font_size: FontSize::Px(32.0) }
			]
			on(|_event: On<Pointer<Click>>, mut game_state: ResMut<NextState<GameState>>| {
				game_state.set(GameState::InitGameplay);
			})
		}
	}
}

fn rotate_fruits(q_fruits: Query<(&mut UiTransform, &SpinningFruit)>, time: Res<Time>) {
	for (mut tf, spin) in q_fruits {
		let rads = time.delta_secs() * spin.speed * if spin.clockwise { 1.0 } else { -1.0 };
		let rotation = Rot2::radians(rads);
		tf.rotation *= rotation;
	}
}
