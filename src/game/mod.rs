use crate::GameState;
use archipelago_rs::Client;
use avian2d::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use validator::Validate;

pub const GAME_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

pub const LEVEL_DIM: Vec2 = Vec2::new(200.0, 600.0);
pub const WALL_PAD: f32 = 64.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_observer(archipelago_error_handler)
			.add_systems(OnEnter(GameState::InitGameplay), (level.spawn(), init_game))
			.add_systems(OnEnter(GameState::NewGame), new_game)
			.add_systems(OnExit(GameState::LoseGame), clear_last_game)
			.add_systems(
				FixedUpdate,
				receive_events.run_if(resource_exists::<ApClient>),
			);
	}
}

fn init_game(mut game_state: ResMut<NextState<GameState>>) {
	game_state.set(GameState::NewGame);
}

fn new_game() {}

fn clear_last_game() {}

fn level() -> impl Scene {
	bsn! {
		Name::new("Level")
		Transform Visibility
		template_value(GAME_RENDER_LAYER)
		Children [
			@AreaBound {
				@name: "Left",
				@location: Vec2 {
					x: { -(LEVEL_DIM.x + WALL_PAD / 2.0) },
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { LEVEL_DIM.y },
				}
			},
			@AreaBound {
				@name: "Right",
				@location: Vec2 {
					x: { LEVEL_DIM.x + WALL_PAD / 2.0 },
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { LEVEL_DIM.y },
				}
			},
			@AreaBound {
				@name: "Bottom",
				@location: Vec2 {
					x: 0.0,
					y: { -( LEVEL_DIM.y / 2.0 + WALL_PAD / 2.0 ) },
				},
				@size: Vec2 {
					x:  { LEVEL_DIM.x * 2.0 + WALL_PAD * 2.0 },
					y:  WALL_PAD
				}
			}
		]
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(AreaBoundProps)]
struct AreaBound;

#[derive(Default)]
struct AreaBoundProps {
	name: String,
	location: Vec2,
	size: Vec2,
}

impl AreaBound {
	fn scene(props: AreaBoundProps) -> impl Scene {
		const WALL_COLOR: Color = Color::srgb_u8(0xff, 0xee, 0xdb);
		bsn! {
			Name::new(props.name)
			template_value(RigidBody::Static)
			template_value(GAME_RENDER_LAYER)
			Sprite {
				color: WALL_COLOR,
				custom_size: { Some(props.size) }
			}
			Transform {
				translation: { Vec3::from((props.location, 1.0)) }
			}
		}
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApClient(Mutex<Client<SessionData>>);

impl ApClient {
	pub fn new(client: Client<SessionData>) -> Self {
		Self(Mutex::new(client))
	}
}

const MAX_FRUITS: usize = 11;

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct SessionData {
	deathlink: bool,
	shuffle_fruit: Option<[usize; MAX_FRUITS]>,
	also_shuffle_sizes: bool,
	#[validate(range(min = 0, max = 10))]
	height_upgrade_count: usize,
	#[validate(range(min = 0, max = 10))]
	cooldown_upgrade_count: usize,
	starting_cooldown: usize,
	goal: Goal,
	scoresanity: bool,
	extra_scoresanity: bool,
	difficulty: Difficulty,
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Goal {
	#[default]
	LargestFruit,
	Score,
	Both,
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Difficulty {
	VeryEasy,
	Easy,
	EasierThanNormal,
	#[default]
	Normal,
	HarderThanNormal,
	Hard,
	VeryHard,
}

fn receive_events(apclient: ResMut<ApClient>, mut commands: Commands) {
	let mut apclient = apclient.lock();
	for event in apclient.update() {
		match event {
			archipelago_rs::Event::Connected => todo!(),
			archipelago_rs::Event::Updated(updated_fields) => todo!(),
			archipelago_rs::Event::Print(print) => todo!(),
			archipelago_rs::Event::ReceivedItems(_) => todo!(),
			archipelago_rs::Event::Error(err) => commands.trigger(ArchipelagoError(err)),
			archipelago_rs::Event::Bounce {
				games,
				slots,
				tags,
				data,
			} => todo!(),
			archipelago_rs::Event::DeathLink {
				games,
				slots,
				tags,
				time,
				server_time,
				cause,
				source,
			} => todo!(),
			archipelago_rs::Event::KeyChanged {
				key,
				old_value,
				new_value,
				player,
			} => todo!(),
		}
	}
}

#[derive(Event, Deref)]
struct ArchipelagoError(archipelago_rs::Error);

fn archipelago_error_handler(event: On<ArchipelagoError>) {
	error!("{}", event.0);
}
