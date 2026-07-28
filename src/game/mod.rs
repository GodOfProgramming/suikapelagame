use crate::{GameState, component_name, ui::RENDER_SCALE, util::despawn_query};
use archipelago_rs::{Client, DeathLinkOptions};
use avian2d::prelude::*;
use bevy::{
	camera::visibility::RenderLayers,
	ecs::{
		bundle::InsertMode,
		system::{RunSystemOnce, SystemParam, entity_command::insert},
	},
	prelude::*,
};
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use enum_map::{EnumMap, enum_map};
use rand::{Rng, RngExt, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{f32::consts::TAU, sync::LazyLock};
use strum::{EnumCount, VariantArray};
use validator::Validate;

pub const GAME_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

const BASE_WIDTH: f32 = 360.0;

pub const WALL_PAD: f32 = 32.0;
pub const WORLD_DIM: Vec2 = Vec2 {
	x: BASE_WIDTH,
	y: BASE_WIDTH * 1.4,
};
const SIDE_POS: f32 = WORLD_DIM.x / 2.0 + WALL_PAD / 2.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<PlaceholderPosition>()
			.init_resource::<LoseHeight>()
			.init_resource::<NextFruit>()
			.init_resource::<StaticAssets>()
			.init_resource::<Score>()
			.init_resource::<FruitDropDelay>()
			.init_resource::<FruitOrder>()
			.init_resource::<MinVelocityFirstDropThresh>()
			.init_gizmo_group::<GameGizmos>()
			.add_message::<ApMessage>()
			.add_observer(archipelago_error_handler)
			.add_observer(on_make_placeholder.run_if(in_state(GameState::Gameplay)))
			.add_observer(on_remove_placeholder.run_if(in_state(GameState::Gameplay)))
			.add_observer(on_new_fruit_body.run_if(in_state(GameState::Gameplay)))
			.add_observer(SpawnFruit::handle.run_if(in_state(GameState::Gameplay)))
			.add_observer(MovePlaceholder::handle.run_if(in_state(GameState::Gameplay)))
			.add_observer(DropPlaceholder::handle.run_if(in_state(GameState::Gameplay)))
			.add_observer(Win::handle)
			.add_observer(Lose::handle)
			.add_observer(SaveGame::handle)
			.add_observer(LoadGame::handle)
			.add_systems(OnEnter(GameState::InitGameplay), (level.spawn(), init_game))
			.add_systems(
				OnEnter(GameState::Gameplay),
				(despawn_query::<With<Fruit>>, new_game).chain(),
			)
			.add_systems(
				First,
				move_placeholder_location.run_if(in_state(GameState::Gameplay)),
			)
			.add_systems(Update, draw_gizmos)
			.add_systems(
				FixedUpdate,
				(
					receive_archipelago_events.run_if(resource_exists::<ApClient>),
					(
						on_collisions,
						remove_invincibility,
						check_if_lose.after(TransformSystems::Propagate),
					)
						.run_if(in_state(GameState::Gameplay)),
				),
			);
	}
}

fn init_game(
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
			@fruit: Fruit::Cherry,
			@x: 0.0,
		}
	});

	let Some(apclient) = apclient else {
		return;
	};

	let session_data = apclient.slot_data();
	if let Some(shuffled) = session_data.shuffle_fruit {
		fruit_order.0 = shuffled;
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

#[derive(Resource)]
struct StaticAssets {
	pop: Handle<Image>,
	click: Handle<AudioSource>,
	fruit_pops: EnumMap<Fruit, Handle<AudioSource>>,
}

impl FromWorld for StaticAssets {
	fn from_world(world: &mut World) -> Self {
		Self {
			pop: world.load_asset("binary/pop.png"),
			click: world.load_asset("binary/audio/click.mp3"),
			fruit_pops: enum_map! {
				Fruit::Cherry => world.load_asset("binary/audio/cherry.mp3"),
				Fruit::Strawberry => world.load_asset("binary/audio/strawberry.mp3"),
				Fruit::Grapes => world.load_asset("binary/audio/grape.mp3"),
				Fruit::Dekopon => world.load_asset("binary/audio/dekopon.mp3"),
				Fruit::Persimmon => world.load_asset("binary/audio/persimmon.mp3"),
				Fruit::Apple => world.load_asset("binary/audio/apple.mp3"),
				Fruit::Pear => world.load_asset("binary/audio/pear.mp3"),
				Fruit::Peach => world.load_asset("binary/audio/peach.mp3"),
				Fruit::Pineapple => world.load_asset("binary/audio/pineapple.mp3"),
				Fruit::Melon => world.load_asset("binary/audio/melon.mp3"),
				Fruit::Watermelon => world.load_asset("binary/audio/watermelon.mp3"),
			},
		}
	}
}

fn on_collisions(
	mut commands: Commands,
	mut messages: MessageReader<CollisionStart>,
	q_fruits: Query<(&Fruit, &Transform)>,
	mut rng: Single<&mut WyRand, With<GlobalRng>>,
	static_assets: Res<StaticAssets>,
	mut score: ResMut<Score>,
	q_is_placeholder: Query<(), With<PlaceholderFruit>>,
	time: Res<Time>,
	fruit_order: Res<FruitOrder>,
) {
	for msg in messages.read() {
		if q_is_placeholder.contains(msg.collider1) || q_is_placeholder.contains(msg.collider2) {
			continue;
		}

		let [Some((fruit_a, tf_a)), Some((fruit_b, tf_b))] =
			[msg.collider1, msg.collider2].map(|e| q_fruits.get(e).ok())
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

		commands.spawn((
			AudioPlayer::new(static_assets.fruit_pops[*fruit_a].clone()),
			PlaybackSettings::DESPAWN,
		));

		commands.entity(msg.collider1).try_despawn();
		commands.entity(msg.collider2).try_despawn();

		if let Some(fruit) = fruit_order.next(*fruit_a) {
			commands.trigger(SpawnFruit(fruit, middle.xy(), time.elapsed_secs()));
		}

		*score += *fruit_a;
	}
}

fn level() -> impl Scene {
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
					y: { -( WORLD_DIM.y / 2.0 - WALL_PAD / 2.0 ) },
				},
				@size: Vec2 {
					x:  { WORLD_DIM.x + WALL_PAD * 2.0 },
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
				translation: { Vec3::from((props.location * RENDER_SCALE, 1.0)) },
				scale: { Vec3::from((props.size * RENDER_SCALE, 1.0)) }
			}
		}
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApClient(Client<SessionData>);

impl ApClient {
	pub fn new(client: Client<SessionData>) -> Self {
		Self(client)
	}
}

unsafe impl Sync for ApClient {}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct SessionData {
	deathlink: bool,
	shuffle_fruit: Option<[Fruit; Fruit::COUNT]>,
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

fn receive_archipelago_events(mut apclient: ResMut<ApClient>, mut commands: Commands) {
	for event in apclient.update() {
		match event {
			archipelago_rs::Event::Connected => {
				commands.queue(ApMessage::new("Client connected"));
				let session_data = apclient.slot_data();
				info!("Session Data: {session_data:#?}");
			}
			archipelago_rs::Event::Updated(updated_fields) => {
				commands.queue(ApMessage::new("Updated"));
			}
			archipelago_rs::Event::Print(print) => {
				commands.queue(ApMessage::new(print));
			}
			archipelago_rs::Event::ReceivedItems(items) => {
				commands.queue(ApMessage::new("Received Items"));
			}
			archipelago_rs::Event::Error(err) => {
				commands.trigger(ArchipelagoError(err));
			}
			archipelago_rs::Event::Bounce {
				games,
				slots,
				tags,
				data,
			} => {
				commands.queue(ApMessage::new("Bounce"));
			}
			archipelago_rs::Event::DeathLink {
				games,
				slots,
				tags,
				time,
				server_time,
				cause,
				source,
			} => {
				commands.queue(ApMessage::new(format!("")));
			}
			archipelago_rs::Event::KeyChanged {
				key,
				old_value,
				new_value,
				player,
			} => {
				commands.queue(ApMessage::new("KeyChanged"));
			}
		}
	}
}

#[derive(Message, Deref)]
pub struct ApMessage(pub String);

impl ApMessage {
	pub fn new(msg: impl ToString) -> Self {
		Self(msg.to_string())
	}
}

impl Command for ApMessage {
	type Out = ();
	fn apply(self, world: &mut World) -> Self::Out {
		world.write_message(self);
	}
}

#[derive(Event, Deref)]
struct ArchipelagoError(archipelago_rs::Error);

fn archipelago_error_handler(event: On<ArchipelagoError>, mut commands: Commands) {
	error!("{}", event.0);
	commands.queue(ApMessage::new(&event.0))
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

#[derive(Resource, Default, Reflect, Deref)]
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
		placeholder: Single<(Entity, &Fruit, &Transform), With<PlaceholderFruit>>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
		mut next_fruit: ResMut<NextFruit>,
		static_assets: Res<StaticAssets>,
		placeholder_pos: Res<PlaceholderPosition>,
		delay: Res<FruitDropDelay>,
		time: Res<Time>,
	) {
		let (entity, fruit, tf) = *placeholder;
		commands
			.entity(entity)
			.remove::<PlaceholderFruit>()
			.apply_scene(bsn! {
				@FruitBody {
					@fruit: { *fruit },
					@position: { tf.translation.xy() }
				}
				Invincible({ time.elapsed_secs() })
			});

		commands.spawn((
			AudioPlayer::new(static_assets.click.clone()),
			PlaybackSettings::DESPAWN,
		));

		let fruit = next_fruit.next(&mut rng);

		commands.delayed().secs(**delay).queue_spawn_scene(bsn! {
			@PlaceholderFruit {
				@fruit: fruit,
				@x: { placeholder_pos.x },
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
			{ component_name!(PlaceholderFruit) }
			{ template_value(props.fruit) }
			Sprite {
				image: { props.fruit.sprite() },
				custom_size: { Some(Vec2::splat(props.fruit.radius())) }
			}
			Transform {
				translation: { Vec3::new(props.x, WORLD_DIM.y * 1.8, 1.0) },
				scale: Vec3::from((Vec2::splat(RENDER_SCALE), 1.0)),
			}
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
			Collider::circle(props.fruit.radius() / 2.0)
			CollisionEventsEnabled
			SweptCcd
			Mass({props.fruit.radius()})
			Restitution {
				coefficient: 0.1,
			}
			Friction {
				dynamic_coefficient: 0.06,
				static_coefficient: 0.06,
			}
			Sprite {
				image: { props.fruit.sprite() },
				custom_size: { Some(Vec2::splat(props.fruit.radius())) }
			}
			Transform {
				translation: { Vec3::from((props.position, 1.0)) },
				scale: Vec3::from((Vec2::splat(RENDER_SCALE), 1.0)),
			}
			template_value(props.fruit)
			template_value(RigidBody::Dynamic)
		}
	}
}

#[derive(Resource, Default, Deref)]
pub struct Score(usize);

impl Score {
	fn restart(&mut self) {
		self.0 = 0;
	}
}

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
			weight: Some(447),
		},
		Fruit::Strawberry => FruitData {
			radius: 32.0,
			value: 3,
			sprite: "binary/fruit/strawberry.png",
			weight: Some(185),
		},
		Fruit::Grapes => FruitData {
			radius: 40.0,
			value: 6,
			sprite: "binary/fruit/grape.png",
			weight: Some(142),
		},
		Fruit::Dekopon => FruitData {
			radius: 56.0,
			value: 10,
			sprite: "binary/fruit/dekopon.png",
			weight: Some(120),
		},
		Fruit::Persimmon => FruitData {
			radius: 64.0,
			value: 15,
			sprite: "binary/fruit/persimmon.png",
			weight: Some(106),
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

fn on_new_fruit_body(event: On<Add, FruitBody>, mut commands: Commands) {
	commands
		.entity(event.event_target())
		.insert(SpeculativeMargin(4.0));
}

#[derive(Event)]
struct SpawnFruit(Fruit, Vec2, f32);

impl SpawnFruit {
	fn handle(event: On<Self>, mut commands: Commands) {
		commands.spawn_scene(bsn! {
			@FruitBody {
				@fruit: {event.0},
				@position: { event.1 },
			}
			Invincible({ event.2 })
		});
	}
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GameGizmos;

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct LoseHeight(f32);

impl Default for LoseHeight {
	fn default() -> Self {
		Self(WORLD_DIM.y * 1.5)
	}
}

#[derive(Event)]
struct Win;

impl Win {
	fn handle(_: On<Self>, mut game_state: ResMut<NextState<GameState>>) {
		game_state.set(GameState::WinGame);
	}
}

#[derive(Event)]
struct Lose;

impl Lose {
	fn handle(
		_: On<Self>,
		mut game_state: ResMut<NextState<GameState>>,
		apclient: Option<ResMut<ApClient>>,
	) -> Result {
		game_state.set(GameState::LoseGame);

		let Some(mut apclient) = apclient else {
			return Ok(());
		};

		let player = apclient.this_player();
		let cause = format!("{} topped out!", player.name());
		apclient.death_link(DeathLinkOptions::new().cause(cause))?;
		Ok(())
	}
}

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
struct FruitDropDelay(f32);

impl Default for FruitDropDelay {
	fn default() -> Self {
		Self(0.5)
	}
}

#[derive(SystemParam)]
struct SaveParams<'w, 's> {
	score: Res<'w, Score>,
	placeholder: Single<'w, 's, &'static Fruit, With<PlaceholderFruit>>,
	next: Res<'w, NextFruit>,
	rng: Single<'w, 's, &'static WyRand, With<GlobalRng>>,
	fruits: Query<
		'w,
		's,
		(
			&'static Fruit,
			&'static Transform,
			&'static LinearVelocity,
			&'static AngularVelocity,
			Option<&'static Invincible>,
		),
		Without<PlaceholderFruit>,
	>,
}

#[derive(Event)]
pub struct SaveGame;

impl SaveGame {
	fn handle(_: On<Self>, mut commands: Commands, params: SaveParams) -> Result {
		let fruits = params
			.fruits
			.iter()
			.map(|f| FruitSaveData {
				fruit: *f.0,
				transform: *f.1,
				linear_velocity: *f.2,
				angular_velocity: *f.3,
				invincibility: f.4.map(|i| i.0),
			})
			.collect();

		let savegame = SaveGameData {
			score: **params.score,
			placeholder: **params.placeholder,
			next: **params.next,
			random_state: params.rng.clone(),
			fruits,
		};

		let out = postcard::to_stdvec(&savegame)?;

		commands.spawn_task(async || {
			let Some(this_dir) = std::env::current_exe()
				.ok()
				.and_then(|f| f.parent().map(|p| p.to_path_buf()))
			else {
				return Ok(());
			};

			if let Err(err) = smol::fs::write(this_dir.join("save.bin"), out).await {
				error!("Failed to save game: {err}");
				return Ok(());
			}

			Ok(())
		});

		Ok(())
	}
}

#[derive(Event)]
pub struct LoadGame;

impl LoadGame {
	fn handle(_: On<Self>, mut commands: Commands, q_fruits: Query<Entity, With<Fruit>>) {
		for fruit in q_fruits {
			commands.entity(fruit).despawn();
		}

		commands.spawn_task(async || {
			let Some(this_dir) = std::env::current_exe()
				.ok()
				.and_then(|f| f.parent().map(|p| p.to_path_buf()))
			else {
				return Ok(());
			};

			let bytes = match smol::fs::read(this_dir.join("save.bin")).await {
				Ok(bytes) => bytes,
				Err(err) => {
					error!("Failed to load game: {err}");
					return Ok(());
				}
			};

			let data = match postcard::from_bytes(&bytes) {
				Ok(data) => data,
				Err(err) => {
					error!("Failed to load game: {err}");
					return Ok(());
				}
			};

			AsyncWorld.apply_command(|world: &mut World| {
				if let Err(err) = world.run_system_once_with(load_game, data) {
					error!("Failed to load game: {err}");
				}
			});

			Ok(())
		});
	}
}

#[derive(Serialize, Deserialize)]
struct SaveGameData {
	score: usize,
	placeholder: Fruit,
	next: Fruit,
	random_state: WyRand,
	fruits: Vec<FruitSaveData>,
}

#[derive(Serialize, Deserialize)]
struct FruitSaveData {
	fruit: Fruit,
	transform: Transform,
	linear_velocity: LinearVelocity,
	angular_velocity: AngularVelocity,
	invincibility: Option<f32>,
}

#[derive(SystemParam)]
struct LoadParams<'w, 's> {
	commands: Commands<'w, 's>,
	score: ResMut<'w, Score>,
	placeholder_pos: Res<'w, PlaceholderPosition>,
	next: ResMut<'w, NextFruit>,
	rng: Single<'w, 's, &'static mut WyRand, With<GlobalRng>>,
}

fn load_game(data: In<SaveGameData>, mut params: LoadParams) {
	params.score.0 = data.score;
	params.next.0 = data.next;
	**params.rng = data.random_state.clone();

	params.commands.queue_spawn_scene(bsn! {
		@PlaceholderFruit {
			@fruit: { data.placeholder },
			@x: { params.placeholder_pos.x },
		}
	});

	for fruit in &data.fruits {
		params.commands.queue_spawn_scene(bsn! {
			@FruitBody {
				@fruit: { fruit.fruit }
			}
			CollisionEventsEnabled
			template_value(fruit.transform)
			template_value(fruit.linear_velocity)
			template_value(fruit.angular_velocity)
			template_value(RigidBody::Dynamic)
			{ fruit.invincibility.map(|i| template_value(Invincible(i))) }
		});
	}
}

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
struct MinVelocityFirstDropThresh(f32);

impl Default for MinVelocityFirstDropThresh {
	fn default() -> Self {
		Self(10.0)
	}
}

#[derive(Component, Default, Clone)]
#[component(storage = "SparseSet")]
struct Invincible(f32);

fn remove_invincibility(
	mut commands: Commands,
	q_invincible: Query<(Entity, &LinearVelocity, &Invincible)>,
	min_thresh: Res<MinVelocityFirstDropThresh>,
	time: Res<Time>,
) {
	let elapsed = time.elapsed_secs();
	for (entity, vel, _) in q_invincible
		.into_iter()
		.filter(|(_, _, inv)| elapsed - inv.0 > 5.0)
	{
		if vel.length() < **min_thresh {
			commands.entity(entity).remove::<Invincible>();
		}
	}
}

fn check_if_lose(
	mut commands: Commands,
	q_tf: Query<(&Transform, &Fruit), (Without<Invincible>, Without<PlaceholderFruit>)>,
	end_height: Res<LoseHeight>,
) {
	for (tf, fruit) in q_tf {
		let y = tf.translation.y + fruit.radius() * 2.0;
		if y >= **end_height {
			commands.trigger(Lose);
			return;
		}
	}
}

#[derive(Resource, Deref)]
pub struct FruitOrder([Fruit; Fruit::COUNT]);

impl Default for FruitOrder {
	fn default() -> Self {
		Self(Fruit::VARIANTS.try_into().unwrap())
	}
}

impl FruitOrder {
	fn next(&self, current: Fruit) -> Option<Fruit> {
		self
			.0
			.iter()
			.enumerate()
			.find_map(|(i, f)| (*f == current).then_some(i))
			.and_then(|i| self.0.get(i + 1))
			.cloned()
	}
}
