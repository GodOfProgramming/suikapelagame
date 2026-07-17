pub mod components;
pub mod start_menu;

use crate::{
	GameCamera, GameState,
	game::{ApClient, DropPlaceholder, LEVEL_DIM, MovePlaceholder},
	ui::{
		components::{next_fruit::NextFruitPlugin, score::ScorePlugin},
		start_menu::MainMenuPlugin,
	},
	util::events::{send_event, send_event_on},
};
use bevy::{
	camera::{ImageRenderTarget, RenderTarget},
	prelude::*,
	render::render_resource::TextureFormat,
	ui_widgets::observe,
};
use components::{FruitCircleComponent, LoginFormComponent, NextFruitComponent, ScoreComponent};
use nameof::name_of;
use start_menu::StartMenuComponent;

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
			.add_observer(DisableLogin::handler)
			.add_observer(send_event_on::<Add, ApClient>(DisableLogin))
			.add_plugins((MainMenuPlugin, NextFruitPlugin, ScorePlugin))
			.add_systems(Startup, GameUi::scene.spawn())
			.add_systems(
				OnExit(GameState::MainMenu),
				(send_event(DisableLogin), clear_center),
			)
			.add_systems(OnEnter(GameState::InitGameplay), init_game_view);
	}
}

#[derive(Event, Clone)]
struct DisableLogin;

impl DisableLogin {
	fn handler(
		_: On<Self>,
		mut commands: Commands,
		q_login_forms: Query<Entity, With<LoginFormComponent>>,
	) {
		for entity in q_login_forms {
			commands.entity(entity).despawn();
		}
	}
}

#[derive(Component, Default, Clone)]
struct GameUi;

impl GameUi {
	fn scene() -> impl Scene {
		bsn! {
			GameUi
			Name::new("Ui Root")
			Transform Visibility
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Row,
				width: percent(100),
				height: percent(100),
				justify_content: JustifyContent::FlexStart,
				align_items: AlignItems::Stretch,
			}
			Children [
				left(),
				@MiddleComponent,
				right(),
			]
			on(|event: On<Add, Self>, mut commands: Commands, game_camera: Single<Entity, With<GameCamera>>| {
				commands.entity(event.event_target()).insert(UiTargetCamera(*game_camera));
			})
		}
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

#[derive(SceneComponent, Default, Clone)]
struct MiddleComponent;

impl MiddleComponent {
	fn scene() -> impl Scene {
		bsn! {
			Name::new("Middle")
			Transform Visibility
			Node {
				width: percent(60),
				height: percent(100),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				aspect_ratio: { Some(1.0 / 3.0) }
			}
			Children [
				@StartMenuComponent
			]
		}
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

fn clear_center(
	mut commands: Commands,
	q_middle_children: Query<&Children, With<MiddleComponent>>,
) {
	for children in q_middle_children {
		for &child in children {
			commands.entity(child).despawn();
		}
	}
}

#[derive(Component, Default, Clone)]
struct GameViewCamera;

#[derive(Component)]
struct GameView;

fn init_game_view(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	q_middle_components: Query<Entity, With<MiddleComponent>>,
) {
	let dim = LEVEL_DIM.as_uvec2();
	for mc in q_middle_components {
		let image = Image::new_target_texture(dim.x, dim.y, TextureFormat::Rgba8UnormSrgb, None);
		let image_handle = images.add(image);

		commands.spawn((
			Name::new(name_of!(GameViewCamera)),
			Camera {
				clear_color: ClearColorConfig::Custom(Color::srgb_u8(0xff, 0xdc, 0xae)),
				..default()
			},
			Camera2d,
			GameViewCamera,
			RenderTarget::Image(ImageRenderTarget {
				handle: image_handle.clone(),
				scale_factor: 1.0,
			}),
			Projection::Orthographic(OrthographicProjection::default_2d()),
		));

		commands.entity(mc).insert((
			Node {
				width: percent(100),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			Children::spawn_one((
				GameView,
				ImageNode::new(image_handle.clone()),
				Node {
					width: percent(60),
					..default()
				},
				observe(|event: On<Pointer<Move>>, mut commands: Commands| {
					let Some(cursor) = event.hit.position else {
						return;
					};

					let pos = cursor.xy().with_y(-cursor.y);

					commands.trigger(MovePlaceholder(pos * LEVEL_DIM));
				}),
				observe(send_event_on::<Pointer<Click>, ()>(DropPlaceholder)),
			)),
		));
	}
}
