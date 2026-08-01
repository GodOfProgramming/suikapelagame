pub mod traps;

use super::FruitKind;
use crate::game::{
	archipelago::traps::{BlindnessTrap, ImpulseTrap, SecrecyTrap, ShuffleTrap, ThanosTrap},
	evt::DropPlaceholder,
	res::Score,
	termination::lose::{Lose, LoseHeight},
};
use archipelago_rs::Client;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::EnumCount;
use strum_macros::FromRepr;
use traps::TrapPlugin;
use validator::Validate;

pub struct ArchipelagoPlugin;

impl Plugin for ArchipelagoPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_message::<ApMessage>()
			.add_plugins(TrapPlugin)
			.add_observer(archipelago_error_handler)
			.add_observer(ReceivedItem::handle)
			.add_observer(ProgressiveHeightItem::handle)
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
			ApItem::UsefulPoints => commands.trigger(BonusPoints(50)),
			ApItem::TrapInstaDrop => commands.trigger(DropPlaceholder),
			ApItem::TrapShuffle => commands.trigger(ShuffleTrap),
			ApItem::TrapImpulse => commands.trigger(ImpulseTrap),
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
struct BonusPoints(usize);

impl BonusPoints {
	fn handle(event: On<Self>, mut score: ResMut<Score>) {
		**score += event.0;
	}
}
