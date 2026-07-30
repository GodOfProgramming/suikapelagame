pub mod archipelago;
pub mod cmp;
pub mod evt;
pub mod persistence;
pub mod res;
pub mod scn;
pub mod termination;

use crate::{
	GameState,
	game::{res::ResPlugin, termination::TerminationPlugin},
	util::despawn_query,
};
use archipelago::{ApClient, ArchipelagoPlugin};
use avian2d::prelude::*;
use bevy::{
	camera::visibility::RenderLayers,
	ecs::{
		bundle::InsertMode,
		system::{SystemParam, entity_command::insert},
	},
	prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use cmp::{FruitValue, Invincible};
use evt::{DropPlaceholder, MovePlaceholder, SpawnFruit};
use persistence::PersistencePlugin;
use rand::RngExt;
use res::{FruitOrder, MinVelocityFirstDropThresh, PlaceholderPosition, Score, StaticAssets};
use scn::{FruitBody, PlaceholderFruit, level};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::f32::consts::TAU;
use termination::lose::LoseHeight;

pub const GAME_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

const BASE_WIDTH: f32 = 360.0;

pub const WORLD_DIM: Vec2 = Vec2 {
	x: BASE_WIDTH,
	y: BASE_WIDTH * 1.4,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.init_gizmo_group::<GameGizmos>()
			.add_plugins((
				ResPlugin,
				ArchipelagoPlugin,
				PersistencePlugin,
				TerminationPlugin,
			))
			.add_observer(on_make_placeholder.run_if(in_state(GameState::Gameplay)))
			.add_observer(on_remove_placeholder.run_if(in_state(GameState::Gameplay)))
			.add_observer(on_new_fruit_body.run_if(in_state(GameState::Gameplay)))
			.add_observer(SpawnFruit::handle.run_if(in_state(GameState::Gameplay)))
			.add_observer(MovePlaceholder::handle.run_if(in_state(GameState::Gameplay)))
			.add_observer(DropPlaceholder::handle.run_if(in_state(GameState::Gameplay)))
			.add_systems(
				OnEnter(GameState::InitGameplay),
				(level.spawn(), init_gameplay),
			)
			.add_systems(
				OnEnter(GameState::Gameplay),
				(despawn_query::<With<FruitKind>>, new_game).chain(),
			)
			.add_systems(
				First,
				move_placeholder_location.run_if(in_state(GameState::Gameplay)),
			)
			.add_systems(Update, draw_gizmos)
			.add_systems(
				FixedUpdate,
				(on_collisions, remove_invincibility).run_if(in_state(GameState::Gameplay)),
			);
	}
}

fn init_gameplay(
	mut game_state: ResMut<NextState<GameState>>,
	mut gizmo_config: ResMut<GizmoConfigStore>,
) {
	game_state.set(GameState::Gameplay);
	let (config, _) = gizmo_config.config_mut::<GameGizmos>();
	config.line.width = 8.0;
	config.line.style = GizmoLineStyle::Dashed {
		gap_scale: 3.0,
		line_scale: 7.0,
	};
}

fn new_game(
	mut commands: Commands,
	mut score: ResMut<Score>,
	mut fruit_order: ResMut<FruitOrder>,
	apclient: Option<Res<ApClient>>,
) {
	score.restart();
	commands.queue_spawn_scene(bsn! {
		@PlaceholderFruit {
			@fruit: FruitKind::Cherry,
			@x: 0.0,
		}
	});

	let Some(apclient) = apclient else {
		return;
	};

	let session_data = apclient.slot_data();
	if let Some(shuffled) = session_data.shuffle_fruit {
		fruit_order.order = shuffled;
	}
}

fn draw_gizmos(mut gizmos: Gizmos<GameGizmos>, end_height: Res<LoseHeight>) {
	let left = Vec2::new(-WORLD_DIM.x * 2.0, **end_height);
	let right = Vec2::new(WORLD_DIM.x * 2.0, **end_height);
	gizmos.line_2d(left, right, Color::BLACK);
}

fn move_placeholder_location(
	q_placeholders: Query<&mut Transform, With<PlaceholderFruit>>,
	location: Res<PlaceholderPosition>,
) {
	for mut tf in q_placeholders {
		tf.translation.x = location.0.x;
	}
}

#[derive(SystemParam)]
struct CollisionRes<'w> {
	static_assets: Res<'w, StaticAssets>,
	score: ResMut<'w, Score>,
	time: Res<'w, Time>,
	fruit_order: Res<'w, FruitOrder>,
}

fn on_collisions(
	mut commands: Commands,
	mut messages: MessageReader<CollisionStart>,
	q_fruits: Query<(&FruitKind, &FruitValue, &Transform)>,
	mut rng: Single<&mut WyRand, With<GlobalRng>>,
	q_is_placeholder: Query<(), With<PlaceholderFruit>>,
	mut res: CollisionRes,
) {
	for msg in messages.read() {
		if q_is_placeholder.contains(msg.collider1) || q_is_placeholder.contains(msg.collider2) {
			continue;
		}

		let [
			Some((fruit_a, value_a, tf_a)),
			Some((fruit_b, value_b, tf_b)),
		] = [msg.collider1, msg.collider2].map(|e| q_fruits.get(e).ok())
		else {
			continue;
		};

		if fruit_a != fruit_b {
			continue;
		}

		let middle = (tf_a.translation + tf_b.translation) / 2.0;

		let pop_entity = commands
			.spawn((
				Sprite {
					image: res.static_assets.pop.clone(),
					custom_size: Some(Vec2::splat(fruit_a.radius())),
					..default()
				},
				Transform {
					translation: Vec3::from((middle.xy(), 1.0)),
					rotation: Quat::from_rotation_z(rng.random_range(0.0..TAU)),
					..default()
				},
			))
			.id();

		commands
			.delayed()
			.secs(0.1)
			.entity(pop_entity)
			.try_despawn();

		commands.spawn((
			AudioPlayer::new(res.static_assets.fruit_pops[*fruit_a].clone()),
			PlaybackSettings::DESPAWN,
		));

		commands.entity(msg.collider1).try_despawn();
		commands.entity(msg.collider2).try_despawn();

		if let Some((fruit, data)) = res.fruit_order.next(*fruit_a) {
			commands.trigger(SpawnFruit {
				fruit,
				data,
				position: middle.xy(),
				invincibility: res.time.elapsed_secs(),
			});
		}

		**res.score += **value_a + **value_b;
	}
}

fn on_make_placeholder(event: On<Add, PlaceholderFruit>, mut commands: Commands) {
	commands
		.entity(event.event_target())
		.insert(RigidBody::Static);
}

fn on_remove_placeholder(
	event: On<Remove, PlaceholderFruit>,
	mut commands: Commands,
	q_fruits: Query<&FruitKind>,
) {
	let Ok(mut entity) = commands.get_spawned_entity(event.event_target()) else {
		return;
	};

	entity.queue_silenced(insert(
		(RigidBody::Dynamic, CollisionEventsEnabled),
		InsertMode::Replace,
	));

	if let Ok(fruit) = q_fruits.get(event.event_target()) {
		entity.queue_silenced(insert(Name::new(fruit.to_string()), InsertMode::Replace));
	}
}

#[derive(
	Component,
	Debug,
	Default,
	Clone,
	Copy,
	PartialEq,
	Eq,
	enum_map::Enum,
	strum_macros::Display,
	strum_macros::VariantArray,
	strum_macros::EnumCount,
	strum_macros::FromRepr,
	strum_macros::EnumIter,
	enum_iterator::Sequence,
	Serialize_repr,
	Deserialize_repr,
)]
#[repr(usize)]
pub enum FruitKind {
	#[default]
	Cherry,
	Strawberry,
	Grapes,
	Dekopon,
	Persimmon,
	Apple,
	Pear,
	Peach,
	Pineapple,
	Melon,
	Watermelon,
}

#[derive(Clone, Copy)]
pub struct FruitData {
	radius: f32,
	value: usize,
	sprite: &'static str,
	weight: Option<usize>,
}

impl FruitKind {
	fn data(self) -> FruitData {
		match self {
			FruitKind::Cherry => FruitData {
				radius: 24.0,
				value: 1,
				sprite: "binary/fruit/cherry.png",
				weight: Some(447),
			},
			FruitKind::Strawberry => FruitData {
				radius: 32.0,
				value: 3,
				sprite: "binary/fruit/strawberry.png",
				weight: Some(185),
			},
			FruitKind::Grapes => FruitData {
				radius: 40.0,
				value: 6,
				sprite: "binary/fruit/grape.png",
				weight: Some(142),
			},
			FruitKind::Dekopon => FruitData {
				radius: 56.0,
				value: 10,
				sprite: "binary/fruit/dekopon.png",
				weight: Some(120),
			},
			FruitKind::Persimmon => FruitData {
				radius: 64.0,
				value: 15,
				sprite: "binary/fruit/persimmon.png",
				weight: Some(106),
			},
			FruitKind::Apple => FruitData {
				radius: 72.0,
				value: 21,
				sprite: "binary/fruit/apple.png",
				weight: None,
			},
			FruitKind::Pear => FruitData {
				radius: 84.0,
				value: 28,
				sprite: "binary/fruit/pear.png",
				weight: None,
			},
			FruitKind::Peach => FruitData {
				radius: 96.0,
				value: 36,
				sprite: "binary/fruit/peach.png",
				weight: None,
			},
			FruitKind::Pineapple => FruitData {
				radius: 128.0,
				value: 45,
				sprite: "binary/fruit/pineapple.png",
				weight: None,
			},
			FruitKind::Melon => FruitData {
				radius: 160.0,
				value: 55,
				sprite: "binary/fruit/melon.png",
				weight: None,
			},
			FruitKind::Watermelon => FruitData {
				radius: 192.0,
				value: 66,
				sprite: "binary/fruit/watermelon.png",
				weight: None,
			},
		}
	}

	pub fn radius(self) -> f32 {
		self.data().radius
	}

	pub fn sprite(self) -> &'static str {
		self.data().sprite
	}

	pub fn weight(self) -> Option<usize> {
		self.data().weight
	}
}

fn on_new_fruit_body(event: On<Add, FruitBody>, mut commands: Commands) {
	commands
		.entity(event.event_target())
		.insert(SpeculativeMargin(4.0));
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GameGizmos;

fn remove_invincibility(
	mut commands: Commands,
	q_invincible: Query<(Entity, &LinearVelocity, &Invincible)>,
	min_thresh: Res<MinVelocityFirstDropThresh>,
	time: Res<Time>,
) {
	let elapsed = time.elapsed_secs();
	for (entity, vel, _) in q_invincible
		.into_iter()
		.filter(|(_, _, inv)| elapsed - ***inv > 5.0)
	{
		if vel.length() < **min_thresh {
			commands.entity(entity).remove::<Invincible>();
		}
	}
}
