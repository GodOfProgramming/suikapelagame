pub mod components;

use crate::{
	GameCamera, GameState,
	game::{
		WORLD_DIM,
		archipelago::ApLoginComplete,
		evt::{DropPlaceholder, MovePlaceholder},
	},
	ui::components::{
		appzone,
		chat::{ChatComponent, ChatPlugin},
		fruit_circle::FruitCirclePlugin,
		login_form,
		lose_popup::LosePopupPlugin,
		next_fruit::NextFruitPlugin,
		saveload::SaveLoadComponent,
		score::ScorePlugin,
		start_menu::{MainMenuPlugin, StartMenuComponent},
		win_popup::WinPopupPlugin,
	},
	util::{
		component_exists, despawn_query,
		events::{send_event, send_event_on},
	},
};
use bevy::{
	camera::{ImageRenderTarget, RenderTarget},
	prelude::*,
	render::render_resource::TextureFormat,
};
use components::{FruitCircleComponent, LoginFormComponent, NextFruitComponent, ScoreComponent};
use nameof::name_of;

#[macro_export]
macro_rules! component_name {
	($name:ident) => {{
		let _: Option<$name> = None;
		bevy::scene::template_value(bevy::ecs::name::Name::new(stringify!($name)))
	}};
}

pub const RENDER_SCALE: f32 = 4.0;

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_observer(DisableLogin::handler)
			.add_observer(send_event_on::<ApLoginComplete, ()>(DisableLogin))
			.add_observer(EnableChatbox::handle)
			.add_plugins((
				MainMenuPlugin,
				NextFruitPlugin,
				ScorePlugin,
				WinPopupPlugin,
				LosePopupPlugin,
				ChatPlugin,
				FruitCirclePlugin,
			))
			.add_systems(Startup, GameUi::scene.spawn())
			.add_systems(
				OnExit(GameState::MainMenu),
				(send_event(DisableLogin), clear_center),
			)
			.add_systems(
				OnEnter(GameState::Gameplay),
				(send_event(EnableChatbox), add_gameplay_ui).chain(),
			)
			.add_systems(
				OnExit(GameState::Gameplay),
				despawn_query::<With<SaveLoadComponent>>,
			)
			.add_systems(OnEnter(GameState::InitGameplay), init_game_view)
			.add_systems(
				FixedUpdate,
				login_form::on_submit.run_if(component_exists::<LoginFormComponent>),
			);
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
				@LeftUi,
				@MiddleUi,
				right(),
			]
			on(|event: On<Add, Self>, mut commands: Commands, game_camera: Single<Entity, With<GameCamera>>| {
				commands.entity(event.event_target()).insert(UiTargetCamera(*game_camera));
			})
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct LeftUi;

#[derive(Component, Default, Clone)]
struct LeftZone;

impl LeftUi {
	fn scene() -> impl Scene {
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
				appzone(bsn! {
					@LoginFormComponent Node {
						width: percent(100),
				}})
				LeftZone Node {
					display: Display::Flex,
					flex_direction: FlexDirection::Column,
					width: percent(100),
					margin: { UiRect::left(px(48)).with_bottom(px(128)) },
				}
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct MiddleUi;

impl MiddleUi {
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

fn clear_center(mut commands: Commands, q_middle_children: Query<&Children, With<MiddleUi>>) {
	for children in q_middle_children {
		for &child in children {
			commands.entity(child).despawn();
		}
	}
}

#[derive(Component, Default, Clone)]
struct GameViewCamera;

#[derive(SceneComponent, Default, Clone)]
#[scene(GameViewComponentProps)]
struct GameViewComponent;

struct GameViewComponentProps {
	parent: Entity,
	image: Handle<Image>,
}

impl Default for GameViewComponentProps {
	fn default() -> Self {
		Self {
			parent: Entity::PLACEHOLDER,
			image: default(),
		}
	}
}

impl GameViewComponent {
	fn scene(props: GameViewComponentProps) -> impl Scene {
		bsn! {
			ImageNode {
				image: { props.image }
			}
			Node {
				width: percent(60),
				max_height: vh(100),
				aspect_ratio: { Some(WORLD_DIM.x / WORLD_DIM.y) },
			}
			ChildOf({ props.parent })
			on(
				|event: On<Pointer<Move>>,
					mut commands: Commands,
					camera: Single<(&Projection, &RenderTarget), With<GameViewCamera>>,
					images: Res<Assets<Image>>| {
					let Some(cursor) = event.hit.position else {
						return;
					};

					let (projection, render_target) = *camera;

					let Projection::Orthographic(ortho) = projection else {
						return;
					};

					let Some(image_handle) = render_target.as_image() else {
						return;
					};

					let Some(image) = images.get(image_handle.id()) else {
						return;
					};

					let pos = cursor.xy().with_y(-cursor.y);

					let world_pos = pos * ortho.scale * image.size().as_vec2();

					commands.trigger(MovePlaceholder(world_pos));
				},
			)
			on(send_event_on::<Pointer<Click>, ()>(DropPlaceholder))
		}
	}
}

fn init_game_view(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	q_middle_components: Query<Entity, With<MiddleUi>>,
) {
	let dim = WORLD_DIM.as_uvec2() * RENDER_SCALE as u32;
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

		commands.queue_spawn_scene(bsn! {
			@GameViewComponent {
				@image: image_handle,
				@parent: mc,
			}
		});
	}
}

#[derive(Event, Clone)]
pub struct EnableChatbox;

impl EnableChatbox {
	fn handle(
		_: On<Self>,
		mut commands: Commands,
		left: Single<Entity, With<LeftZone>>,
		q_chat_boxes: Query<(), With<ChatComponent>>,
	) {
		if !q_chat_boxes.is_empty() {
			return;
		}

		commands.queue_spawn_scene_list(bsn_list! [
			@ChatComponent Node {
				width: percent(100)
			} ChildOf({*left}),
		]);
	}
}

fn add_gameplay_ui(
	mut commands: Commands,
	left: Single<Entity, With<LeftZone>>,
	q_saveload: Query<(), With<SaveLoadComponent>>,
) {
	if !q_saveload.is_empty() {
		return;
	}

	commands.queue_spawn_scene_list(bsn_list! [
		@SaveLoadComponent Node {
			width: percent(100)
		} ChildOf({*left}),
	]);
}
