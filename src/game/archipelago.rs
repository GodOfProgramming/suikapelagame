use crate::{
	component_name,
	game::{
		Fruit, WORLD_DIM,
		evt::{DropPlaceholder, PlaceholderDropped},
		res::FruitOrder,
		scn::PlaceholderFruit,
		termination::lose::{Lose, LoseHeight},
	},
	ui::components::NextFruitComponent,
};

use super::FruitKind;
use archipelago_rs::Client;
use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use rand::seq::{IteratorRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::EnumCount;
use strum_macros::FromRepr;
use validator::Validate;

pub struct ArchipelagoPlugin;

impl Plugin for ArchipelagoPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_message::<ApMessage>()
			.init_resource::<SecrecyTurns>()
			.init_resource::<BlindnessTurns>()
			.add_observer(archipelago_error_handler)
			.add_observer(ReceivedItem::handle)
			.add_observer(ProgressiveHeightItem::handle)
			.add_observer(ShuffleTrap::handle)
			.add_observer(ThanosTrap::handle)
			.add_observer(SecrecyTrap::handle)
			.add_observer(BlindnessTrap::handle)
			.add_observer(on_drop_placeholder)
			.add_systems(
				FixedUpdate,
				receive_archipelago_events.run_if(resource_exists::<ApClient>),
			);
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
	pub deathlink: bool,
	pub shuffle_fruit: Option<[FruitKind; FruitKind::COUNT]>,
	pub also_shuffle_sizes: bool,
	#[validate(range(min = 0, max = 10))]
	pub height_upgrade_count: usize,
	#[validate(range(min = 0, max = 10))]
	pub cooldown_upgrade_count: usize,
	pub starting_cooldown: usize,
	pub goal: Goal,
	pub scoresanity: bool,
	pub extra_scoresanity: bool,
	pub difficulty: Difficulty,
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

fn receive_archipelago_events(
	mut apclient: ResMut<ApClient>,
	mut commands: Commands,
	mut receiving_new_items: Local<bool>,
) {
	for event in apclient.update() {
		match event {
			archipelago_rs::Event::Connected => {
				commands.queue(ApMessage::new("Client connected"));
				let session_data = apclient.slot_data();
				info!("Session Data: {session_data:#?}");
			}
			archipelago_rs::Event::Updated(updated_fields) => {}
			archipelago_rs::Event::Print(print) => {
				commands.queue(ApMessage::new(print));
			}
			archipelago_rs::Event::ReceivedItems(items) => {
				if items == 0 {
					*receiving_new_items = true;
					continue;
				}

				let Some(item) = ApItem::from_repr(items) else {
					error!("Received unknown item {items}");
					return;
				};

				commands.trigger(ReceivedItem(item));
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
				commands.queue(ApMessage::new(format!("DeathLink received from {source}")));
				commands.trigger(Lose);
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

#[derive(FromRepr)]
#[repr(usize)]
enum ApItem {
	ProgressiveHeight = 1,
	ProgressiveFruit = 2,
	ProgressiveNext = 3,
	UsefulDropCooldown = 4,
	UsefulPoints = 5,
	TrapInstaDrop = 6,
	TrapShuffle = 7,
	TrapImpulse = 8,
	TrapThanos = 9,
	TrapSecrecy = 10,
	TrapBlind = 11,
	Smoothie = 12,
}

#[derive(Event)]
struct ReceivedItem(ApItem);

impl ReceivedItem {
	fn handle(event: On<Self>, mut commands: Commands) {
		match event.0 {
			ApItem::ProgressiveHeight => commands.trigger(ProgressiveHeightItem(1)),
			ApItem::ProgressiveFruit => (),
			ApItem::ProgressiveNext => (),
			ApItem::UsefulDropCooldown => (),
			ApItem::UsefulPoints => (),
			ApItem::TrapInstaDrop => commands.trigger(DropPlaceholder),
			ApItem::TrapShuffle => commands.trigger(ShuffleTrap),
			ApItem::TrapImpulse => (),
			ApItem::TrapThanos => commands.trigger(ThanosTrap),
			ApItem::TrapSecrecy => commands.trigger(SecrecyTrap(3)),
			ApItem::TrapBlind => commands.trigger(BlindnessTrap(3)),
			ApItem::Smoothie => (),
		}
	}
}

#[derive(Event)]
struct ProgressiveHeightItem(usize);

impl ProgressiveHeightItem {
	fn handle(event: On<Self>, mut height: ResMut<LoseHeight>) {
		height.add_upgrades(event.0);
	}
}

#[derive(Event)]
struct ShuffleTrap;

impl ShuffleTrap {
	fn handle(
		_: On<Self>,
		mut fruit_order: ResMut<FruitOrder>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
	) {
		fruit_order.order.shuffle(&mut rng);
	}
}

#[derive(Event)]
struct ThanosTrap;

impl ThanosTrap {
	fn handle(
		_: On<Self>,
		mut commands: Commands,
		q_fruits: Query<Entity, (With<Fruit>, Without<PlaceholderFruit>)>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
	) {
		let mut entities = Vec::from_iter(q_fruits.iter());

		for _ in 0..entities.len() / 2 {
			let Some((i, &entity)) = entities.iter().enumerate().choose(&mut rng) else {
				continue;
			};

			entities.swap_remove(i);

			commands.entity(entity).despawn();
		}
	}
}

#[derive(Event)]
pub struct SecrecyTrap(pub usize);

impl SecrecyTrap {
	fn handle(
		event: On<Self>,
		mut turns: ResMut<SecrecyTurns>,
		q_to_hide: Query<&mut Visibility, Or<(With<NextFruitComponent>, With<PlaceholderFruit>)>>,
	) {
		turns.0 = event.0;
		for mut vis in q_to_hide {
			*vis = Visibility::Hidden;
		}
	}
}

#[derive(Resource, Reflect, Default, Deref)]
#[reflect(Resource)]
pub struct SecrecyTurns(usize);

#[derive(Event)]
pub struct BlindnessTrap(pub usize);

impl BlindnessTrap {
	fn handle(
		event: On<Self>,
		mut commands: Commands,
		end_height: Res<LoseHeight>,
		mut blindness: ResMut<BlindnessTurns>,
		blindness_cover: Query<(), With<BlindnessCover>>,
	) {
		blindness.0 = event.0;

		if !blindness_cover.is_empty() {
			return;
		}

		let height = end_height.height();
		let cover = Rect::new(
			-WORLD_DIM.x * 2.0,
			-WORLD_DIM.y * 2.0,
			WORLD_DIM.x * 2.0,
			height,
		);
		let size = cover.size();
		let center = cover.center();
		commands.spawn_scene(bsn! {
			@BlindnessCover {
				@size: size,
				@shift: { center.y },
			}
		});
	}
}

#[derive(Resource, Reflect, Default, Deref)]
#[reflect(Resource)]
pub struct BlindnessTurns(usize);

#[derive(SceneComponent, Default, Clone)]
#[scene(BlindnessCoverParams)]
struct BlindnessCover;

#[derive(Default)]
struct BlindnessCoverParams {
	size: Vec2,
	shift: f32,
}

impl BlindnessCover {
	fn scene(params: BlindnessCoverParams) -> impl Scene {
		bsn! {
			{ component_name!(BlindnessCover) }
			Sprite {
				color: Color::BLACK,
				custom_size: { params.size },
			}
			Transform {
				translation: { Vec3::new(0.0, params.shift, 100.0) }
			}
		}
	}
}

fn on_drop_placeholder(
	_: On<PlaceholderDropped>,
	mut commands: Commands,
	mut secrecy: ResMut<SecrecyTurns>,
	mut blindness: ResMut<BlindnessTurns>,
	q_to_hide: Query<&mut Visibility, Or<(With<NextFruitComponent>, With<PlaceholderFruit>)>>,
	q_covers: Query<Entity, With<BlindnessCover>>,
) {
	// last secrecy turn
	if secrecy.0 == 1 {
		for mut vis in q_to_hide {
			*vis = Visibility::Inherited;
		}
	}

	if blindness.0 == 1 {
		for entity in q_covers {
			commands.entity(entity).despawn();
		}
	}

	secrecy.0 = secrecy.0.saturating_sub(1);
	blindness.0 = blindness.0.saturating_sub(1);
}
