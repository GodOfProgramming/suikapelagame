use std::{f32::consts::TAU, sync::LazyLock};

use crate::{GameState, component_name};
use archipelago_rs::Client;
use avian2d::prelude::*;
use bevy::{
	camera::visibility::RenderLayers,
	ecs::{bundle::InsertMode, entity, system::entity_command::insert},
	prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use enum_map::{EnumMap, enum_map};
use parking_lot::Mutex;
use rand::{Rng, RngExt, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::VariantArray;
use validator::Validate;

pub const GAME_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

pub const LEVEL_DIM: Vec2 = Vec2::new(1200.0, 3600.0);
pub const WALL_PAD: f32 = 32.0;
const WORLD_DIM: Vec2 = Vec2 {
	x: LEVEL_DIM.x / 4.0,
	y: LEVEL_DIM.y / 4.0,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<PlaceholderPosition>()
			.init_resource::<NextFruit>()
			.init_resource::<StaticAssets>()
			.init_resource::<Score>()
			.add_observer(archipelago_error_handler)
			.add_observer(on_make_placeholder)
			.add_observer(on_remove_placeholder)
			.add_observer(MovePlaceholder::handle)
			.add_observer(DropPlaceholder::handle)
			.add_systems(OnEnter(GameState::InitGameplay), (level.spawn(), init_game))
			.add_systems(OnEnter(GameState::NewGame), new_game)
			.add_systems(OnExit(GameState::LoseGame), clear_last_game)
			.add_systems(First, move_placeholder_location)
			.add_systems(
				FixedUpdate,
				(
					receive_events.run_if(resource_exists::<ApClient>),
					on_collisions,
				),
			);
	}
}

fn init_game(mut game_state: ResMut<NextState<GameState>>) {
	game_state.set(GameState::NewGame);
}

fn new_game(mut commands: Commands) {
	commands.queue_spawn_scene(bsn! {
		@PlaceholderFruit {
			@fruit: Fruit::Cherry,
			@x: 0.0,
		}
	});
}

fn clear_last_game(mut commands: Commands, q_fruits: Query<Entity, With<Fruit>>) {
	for entity in q_fruits {
		commands.entity(entity).despawn();
	}
}

fn move_placeholder_location(
	q_placeholders: Query<&mut Transform, With<PlaceholderFruit>>,
	location: Res<PlaceholderPosition>,
) {
	for mut tf in q_placeholders {
		tf.translation.x = location.0.x;
	}
}

#[derive(Resource)]
struct StaticAssets {
	pop: Handle<Image>,
}

impl FromWorld for StaticAssets {
	fn from_world(world: &mut World) -> Self {
		let pop = world.load_asset("binary/pop.png");
		Self { pop }
	}
}

fn on_collisions(
	mut commands: Commands,
	mut messages: MessageReader<CollisionStart>,
	q_fruits: Query<(&Fruit, &Transform)>,
	mut rng: Single<&mut WyRand, With<GlobalRng>>,
	static_assets: Res<StaticAssets>,
	mut score: ResMut<Score>,
) {
	for msg in messages.read() {
		let [Some((fruit_a, tf_a)), Some((fruit_b, tf_b))] =
			[msg.collider1, msg.collider2].map(|e| q_fruits.get(e).ok())
		else {
			return;
		};

		if fruit_a != fruit_b {
			return;
		}

		let middle = (tf_a.translation + tf_b.translation) / 2.0;

		let pop_entity = commands
			.spawn((
				Sprite {
					image: static_assets.pop.clone(),
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

		if let Some(fruit) = fruit_a.next() {
			commands.queue_spawn_scene(bsn! {
				@FruitBody {
					@fruit: fruit,
					@position: { middle.xy() },
				}
				template_value(RigidBody::Dynamic)
				CollisionEventsEnabled
			});
		}

		*score += *fruit_a;

		commands.entity(msg.collider1).try_despawn();
		commands.entity(msg.collider2).try_despawn();
	}
}

fn level() -> impl Scene {
	const SIDE_POS: f32 = LEVEL_DIM.x / 2.0 + WALL_PAD / 2.0;

	bsn! {
		Name::new("Level")
		Transform Visibility
		Children [
			@AreaBound {
				@name: "Left",
				@location: Vec2 {
					x: { -SIDE_POS },
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { WORLD_DIM.y },
				}
			},
			@AreaBound {
				@name: "Right",
				@location: Vec2 {
					x: SIDE_POS,
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { WORLD_DIM.y },
				}
			},
			@AreaBound {
				@name: "Bottom",
				@location: Vec2 {
					x: 0.0,
					y: { -( WORLD_DIM.y / 2.0 + WALL_PAD / 2.0 ) },
				},
				@size: Vec2 {
					x:  { LEVEL_DIM.x + WALL_PAD * 2.0 },
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
			Collider::rectangle(1.0, 1.0)
			Sprite {
				color: WALL_COLOR,
			}
			Transform {
				translation: { Vec3::from((props.location, 1.0)) },
				scale: { Vec3::from((props.size, 1.0)) }
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

fn on_make_placeholder(event: On<Add, PlaceholderFruit>, mut commands: Commands) {
	commands
		.entity(event.event_target())
		.insert(RigidBody::Static);
}

fn on_remove_placeholder(
	event: On<Remove, PlaceholderFruit>,
	mut commands: Commands,
	q_fruits: Query<&Fruit>,
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

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct PlaceholderPosition(Vec2);

#[derive(Event, Clone, Deref)]
pub struct MovePlaceholder(pub Vec2);

impl MovePlaceholder {
	fn handle(event: On<Self>, mut placeholder_location: ResMut<PlaceholderPosition>) {
		placeholder_location.0 = event.0.xy();
	}
}

#[derive(Event, Clone)]
pub struct DropPlaceholder;

impl DropPlaceholder {
	fn handle(
		_: On<Self>,
		mut commands: Commands,
		q_placeholders: Query<Entity, With<PlaceholderFruit>>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
		mut next_fruit: ResMut<NextFruit>,
	) {
		for fruit in q_placeholders {
			commands
				.entity(fruit)
				.remove::<PlaceholderFruit>()
				.insert((RigidBody::Dynamic, LinearVelocity(Vec2::new(0.0, -128.0))));
		}

		let fruit = next_fruit.next(&mut rng);

		commands.delayed().secs(0.5).queue_spawn_scene(bsn! {
			@PlaceholderFruit {
				@fruit: fruit,
				@x: 0.0,
			}
		});
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(PlaceholderFruitProps)]
struct PlaceholderFruit;

#[derive(Default)]
struct PlaceholderFruitProps {
	fruit: Fruit,
	x: f32,
}

impl PlaceholderFruit {
	fn scene(props: PlaceholderFruitProps) -> impl Scene {
		bsn! {
			@FruitBody {
				@fruit: { props.fruit },
				@position: { Vec2::new(props.x, WORLD_DIM.y / 2.0 - WALL_PAD) },
			}
			{ component_name!(PlaceholderFruit) }
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(FruitBodyProps)]
struct FruitBody;

#[derive(Default)]
struct FruitBodyProps {
	fruit: Fruit,
	position: Vec2,
}

impl FruitBody {
	fn scene(props: FruitBodyProps) -> impl Scene {
		bsn! {
			Name::new(props.fruit.to_string())
			Collider::circle(props.fruit.radius())
			template_value(props.fruit)
			Restitution {
				coefficient: 0.1,
			}
			Friction {
				dynamic_coefficient: 0.006,
				static_coefficient: 0.006,
			}
			Sprite {
				image: { props.fruit.sprite() },
				custom_size: { Some(Vec2::splat(props.fruit.radius() * 2.0)) }
			}
			Transform {
				translation: { Vec3::from((props.position, 1.0)) },
			}
		}
	}
}

#[derive(Resource, Default, Deref)]
pub struct Score(usize);

impl std::ops::AddAssign<Fruit> for Score {
	fn add_assign(&mut self, rhs: Fruit) {
		self.0 += rhs.value();
	}
}

#[derive(Resource, Default, Deref)]
pub struct NextFruit(Fruit);

impl NextFruit {
	fn next(&mut self, mut rng: impl Rng) -> Fruit {
		let next_fruit = Fruit::VARIANTS
			.iter()
			.cloned()
			.filter_map(|f| f.weight().map(|w| (f, w)))
			.collect::<Vec<_>>()
			.choose_weighted(&mut rng, |c| c.1)
			.map(|(f, _)| *f)
			.unwrap_or_default();
		let prev = self.0;
		self.0 = next_fruit;
		prev
	}
}

#[derive(
	Component,
	Default,
	Clone,
	Copy,
	PartialEq,
	Eq,
	enum_map::Enum,
	strum_macros::Display,
	strum_macros::VariantArray,
	enum_iterator::Sequence,
)]
pub enum Fruit {
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

struct FruitData {
	radius: f32,
	value: usize,
	sprite: &'static str,
	weight: Option<usize>,
}

static DATA_MAP: LazyLock<EnumMap<Fruit, FruitData>> = LazyLock::new(|| {
	enum_map! {
		Fruit::Cherry => FruitData {
			radius: 24.0,
			value: 1,
			sprite: "binary/fruit/cherry.png",
			weight: Some(7),
		},
		Fruit::Strawberry => FruitData {
			radius: 32.0,
			value: 3,
			sprite: "binary/fruit/strawberry.png",
			weight: Some(5),
		},
		Fruit::Grapes => FruitData {
			radius: 40.0,
			value: 6,
			sprite: "binary/fruit/grape.png",
			weight: Some(3),
		},
		Fruit::Dekopon => FruitData {
			radius: 56.0,
			value: 10,
			sprite: "binary/fruit/dekopon.png",
			weight: Some(2),
		},
		Fruit::Persimmon => FruitData {
			radius: 64.0,
			value: 15,
			sprite: "binary/fruit/persimmon.png",
			weight: Some(1),
		},
		Fruit::Apple => FruitData {
			radius: 72.0,
			value: 21,
			sprite: "binary/fruit/apple.png",
			weight: None,
		},
		Fruit::Pear => FruitData {
			radius: 84.0,
			value: 28,
			sprite: "binary/fruit/pear.png",
			weight: None,
		},
		Fruit::Peach => FruitData {
			radius: 96.0,
			value: 36,
			sprite: "binary/fruit/peach.png",
			weight: None,
		},
		Fruit::Pineapple => FruitData {
			radius: 128.0,
			value: 45,
			sprite: "binary/fruit/pineapple.png",
			weight: None,
		},
		Fruit::Melon => FruitData {
			radius: 160.0,
			value: 55,
			sprite: "binary/fruit/melon.png",
			weight: None,
		},
		Fruit::Watermelon => FruitData {
			radius: 192.0,
			value: 66,
			sprite: "binary/fruit/watermelon.png",
			weight: None,
		},
	}
});

impl Fruit {
	pub fn radius(self) -> f32 {
		DATA_MAP[self].radius
	}

	pub fn value(self) -> usize {
		DATA_MAP[self].value
	}

	pub fn sprite(self) -> &'static str {
		DATA_MAP[self].sprite
	}

	pub fn weight(self) -> Option<usize> {
		DATA_MAP[self].weight
	}

	pub fn next(self) -> Option<Self> {
		enum_iterator::next(&self)
	}
}
