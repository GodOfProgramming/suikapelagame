pub mod traps;

use super::FruitKind;
use crate::{
	GameState,
	game::{
		archipelago::traps::{
			BlindnessTrap, ImpulseTrap, SecrecyTrap, SecrecyTurns, ShuffleTrap, ThanosTrap,
		},
		evt::{DropPlaceholder, FruitCombo, SpawnFruit},
		persistence::LoadSavedGame,
		res::{CanSeeNextFruit, FruitDropCooldown, FruitDropDelay, FruitOrder, Score},
		termination::{
			lose::{Lose, LoseHeight},
			win::Win,
		},
	},
	ui::{
		EnableChatbox,
		components::{
			NextFruitComponent, ScoreComponent,
			chat::msg::{ChatMessage, RichText, TextNode, style_from_apclr},
		},
	},
	util::component_added,
};
use archipelago_rs::{ClientStatus, Connection, ItemHandling};
use bevy::{platform::collections::HashSet, prelude::*};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::EnumCount;
use strum_macros::FromRepr;
use traps::TrapPlugin;
use validator::Validate;

pub struct ArchipelagoPlugin;

impl Plugin for ArchipelagoPlugin {
	fn build(&self, app: &mut App) {
		fn is_logged_in(apclient: Res<ApClient>) -> bool {
			apclient.is_connected()
		}

		app
			.init_resource::<NextThreshold>()
			.init_resource::<NextFruitUnlocked>()
			.init_resource::<ApClient>()
			.add_message::<ChatMessage>()
			.add_plugins(TrapPlugin)
			.add_observer(ReceivedItem::handle.run_if(is_logged_in))
			.add_observer(ProgressiveHeightItem::handle.run_if(is_logged_in))
			.add_observer(BonusPoints::handle.run_if(is_logged_in))
			.add_observer(CooldownUpgrade::handle.run_if(is_logged_in))
			.add_observer(FruitSizeUpgrade::handle.run_if(is_logged_in))
			.add_observer(NextFruitUnlock::handle.run_if(is_logged_in))
			.add_observer(check_spawned_fruit.run_if(is_logged_in))
			.add_observer(on_win.run_if(is_logged_in))
			.add_observer(on_load_savegame.run_if(is_logged_in))
			.add_observer(on_fruit_combo.run_if(is_logged_in))
			.add_observer(on_aplogin)
			.add_systems(
				Update,
				(
					update_cooldown.run_if(in_state(GameState::Gameplay)),
					set_next_scores
						.run_if(resource_changed::<NextThreshold>.or_else(component_added::<NextThreshMarker>)),
				)
					.run_if(is_logged_in),
			)
			.add_systems(
				FixedUpdate,
				(
					receive_archipelago_events,
					(
						monitor_score.run_if(in_state(GameState::Gameplay)),
						monitor_next_fruit_vis.run_if(resource_changed::<CanSeeNextFruit>),
					)
						.run_if(is_logged_in),
				),
			);
	}
}

#[derive(Default, Resource, Deref, DerefMut)]
pub struct ApClient(Connection<SlotData>);

impl ApClient {
	pub fn login(url: String, name: String, pass: Option<String>) -> Self {
		let mut opts =
			archipelago_rs::ConnectionOptions::new().receive_items(ItemHandling::OtherWorlds {
				own_world: true,
				starting_inventory: true,
			});

		if let Some(pass) = pass {
			opts = opts.password(pass);
		}

		Self(Connection::new(url, name, Some("Suikapelagame"), opts))
	}
}

unsafe impl Sync for ApClient {}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct SlotData {
	pub deathlink: bool,
	pub shuffle_fruit: Option<[FruitKind; FruitKind::COUNT]>,
	pub also_shuffle_sizes: bool,
	#[validate(range(min = 0, max = 10))]
	pub height_upgrade_count: usize,
	#[validate(range(min = 0, max = 10))]
	pub cooldown_upgrade_count: usize,
	pub starting_cooldown: usize,
	pub goal: Goal,
	pub scoresanity_thresholds: Vec<ScoresanityThreshold>,
	pub difficulty: Difficulty,
	#[validate(range(min = 1, max = 11))]
	pub max_fruit_size: usize,
	pub next_needs_unlock: bool,
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Goal {
	#[default]
	LargestFruit,
	Score,
	Both,
}

impl Goal {
	pub fn win_by_score(&self) -> bool {
		matches!(self, Self::Score | Self::Both)
	}

	pub fn win_by_fruit(&self) -> bool {
		matches!(self, Self::LargestFruit | Self::Both)
	}
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Difficulty {
	VeryEasy,
	Easy,
	EasierThanNormal,
	#[default]
	Normal,
	HarderThanNormal,
	Hard,
	VeryHard,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ScoresanityThreshold {
	value: usize,
	location: i64,
}

fn receive_archipelago_events(mut apclient: ResMut<ApClient>, mut commands: Commands) {
	for event in apclient.update() {
		match event {
			archipelago_rs::Event::Connected => {
				commands.trigger(ApLoginComplete);

				let Some(apclient) = apclient.client() else {
					continue;
				};

				for item in apclient.received_items() {
					let id = item.item().id();
					let Some(item_type) = ApItem::from_repr(id) else {
						error!("Received unknown item {id}");
						return;
					};

					if item_type.received_on_startup() {
						commands.trigger(ReceivedItem {
							item: item_type,
							location: item.location().id(),
						});
					}
				}
			}
			archipelago_rs::Event::Updated(_) => {}
			archipelago_rs::Event::Print(print) => {
				let Some(apclient) = apclient.client() else {
					continue;
				};
				commands.write_message(ChatMessage::from_print(
					&apclient.this_player().name(),
					print,
				));
			}
			archipelago_rs::Event::ReceivedItems(index) => {
				if index == 0 {
					continue;
				}

				let Some(apclient) = apclient.client() else {
					continue;
				};

				let items = apclient.received_items();

				for item in &items[index..] {
					let id = item.item().id();
					let Some(item_type) = ApItem::from_repr(id) else {
						error!("Received unknown item {id}");
						return;
					};

					commands.trigger(ReceivedItem {
						item: item_type,
						location: item.location().id(),
					});
				}
			}
			archipelago_rs::Event::Error(err) => {
				commands.write_message(ChatMessage::error(err));
			}
			archipelago_rs::Event::Bounce { .. } => {}
			archipelago_rs::Event::DeathLink { source, .. } => {
				commands.write_message(ChatMessage::Text(format!(
					"DeathLink received from {source}"
				)));
				commands.trigger(Lose);
			}
			archipelago_rs::Event::KeyChanged { .. } => {}
		}
	}
}

#[derive(FromRepr, strum_macros::Display)]
#[repr(i64)]
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

impl ApItem {
	fn received_on_startup(&self) -> bool {
		matches!(
			self,
			Self::ProgressiveHeight
				| Self::ProgressiveFruit
				| Self::ProgressiveNext
				| Self::UsefulDropCooldown
		)
	}
}

#[derive(Event)]
struct ReceivedItem {
	item: ApItem,
	location: i64,
}

impl ReceivedItem {
	fn handle(event: On<Self>, mut commands: Commands, mut locations_received: Local<HashSet<i64>>) {
		if locations_received.contains(&event.location) {
			return;
		}

		locations_received.insert(event.location);
		match event.item {
			ApItem::ProgressiveHeight => commands.trigger(ProgressiveHeightItem(1)),
			ApItem::ProgressiveFruit => commands.trigger(FruitSizeUpgrade(1)),
			ApItem::ProgressiveNext => commands.trigger(NextFruitUnlock),
			ApItem::UsefulDropCooldown => commands.trigger(CooldownUpgrade(1)),
			ApItem::UsefulPoints => commands.trigger(BonusPoints(50)),
			ApItem::TrapInstaDrop => commands.trigger(DropPlaceholder),
			ApItem::TrapShuffle => commands.trigger(ShuffleTrap),
			ApItem::TrapImpulse => commands.trigger(ImpulseTrap),
			ApItem::TrapThanos => commands.trigger(ThanosTrap),
			ApItem::TrapSecrecy => commands.trigger(SecrecyTrap(3)),
			ApItem::TrapBlind => commands.trigger(BlindnessTrap(3)),
			ApItem::Smoothie => {
				commands.write_message(smoothie());
			}
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

#[derive(Event)]
struct CooldownUpgrade(usize);

impl CooldownUpgrade {
	fn handle(event: On<Self>, mut drop_delay: ResMut<FruitDropDelay>) {
		drop_delay.add_upgrades(event.0);
	}
}

#[derive(Event)]
struct FruitSizeUpgrade(usize);

impl FruitSizeUpgrade {
	fn handle(event: On<Self>, mut fruit_order: ResMut<FruitOrder>) {
		fruit_order.limit += event.0;
	}
}

#[derive(Event)]
struct NextFruitUnlock;

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
struct NextFruitUnlocked(bool);

impl NextFruitUnlock {
	fn handle(
		_: On<Self>,
		secrecy: Res<SecrecyTurns>,
		mut next_unlocked: ResMut<NextFruitUnlocked>,
		mut can_see_next: ResMut<CanSeeNextFruit>,
	) {
		**next_unlocked = true;

		if secrecy.inactive() {
			**can_see_next = true;
		}
	}
}

#[derive(Event)]
pub struct ApLoginComplete;

fn on_aplogin(
	_: On<ApLoginComplete>,
	mut commands: Commands,
	q_scores: Query<Entity, With<ScoreComponent>>,
	apclient: Res<ApClient>,
	mut fruit_order: ResMut<FruitOrder>,
	mut lose_height: ResMut<LoseHeight>,
	mut drop_delay: ResMut<FruitDropDelay>,
	mut next_unlocked: ResMut<NextFruitUnlocked>,
	mut can_see_next: ResMut<CanSeeNextFruit>,
	mut next_thresh: ResMut<NextThreshold>,
	game_state: Res<State<GameState>>,
	mut next_game_state: ResMut<NextState<GameState>>,
	mut previously_logged_in: Local<bool>,
) {
	if !*previously_logged_in {
		commands.trigger(EnableChatbox);

		for entity in q_scores {
			commands.queue_spawn_scene_list(bsn_list! [
				ChildOf(entity) next(),
				ChildOf(entity) drop_cooldown(),
			]);
		}

		if **game_state == GameState::MainMenu {
			next_game_state.set(GameState::InitGameplay);
		}

		*previously_logged_in = true;
	}

	let Some(apclient) = apclient.client() else {
		unreachable!("Logic error");
	};

	let slot_data = apclient.slot_data();
	if let Some(shuffled) = slot_data.shuffle_fruit {
		fruit_order.set_order(shuffled, slot_data.also_shuffle_sizes);
	}

	lose_height.max_upgrades = slot_data.height_upgrade_count;

	drop_delay.additional_cooldown = slot_data.starting_cooldown as f32;
	drop_delay.max_upgrades = slot_data.cooldown_upgrade_count;

	fruit_order.limit = slot_data.max_fruit_size;

	**next_unlocked = !slot_data.next_needs_unlock;
	**can_see_next = **next_unlocked;

	let Some(next) = get_next_score(
		&slot_data.scoresanity_thresholds,
		0,
		apclient.unchecked_locations(),
	) else {
		return;
	};

	next_thresh.0 = next.value;
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct NextThreshold(usize);

fn monitor_score(
	mut commands: Commands,
	score: Res<Score>,
	mut apclient: ResMut<ApClient>,
	mut index: Local<usize>,
	mut next_threshold: ResMut<NextThreshold>,
) {
	let Some(apclient) = apclient.client_mut() else {
		return;
	};

	let slot_data = apclient.slot_data();

	if slot_data.goal.win_by_score() && *index >= slot_data.scoresanity_thresholds.len() {
		commands.trigger(Win);
	}

	let Some(&thresh) = apclient.slot_data().scoresanity_thresholds.get(*index) else {
		return;
	};

	if thresh.value < **score {
		let res = apclient.mark_checked([thresh.location]);
		if let Err(err) = res {
			error!(err = err.to_string(), "Failed to check location");
		} else {
			*index += 1;
			if let Some(t) = apclient.slot_data().scoresanity_thresholds.get(*index) {
				next_threshold.0 = t.value;
			}
		}
	}
}

fn set_next_scores(
	q_next_thresh: Query<&mut Text, With<NextThreshMarker>>,
	next_thresh: Res<NextThreshold>,
) {
	for mut t in q_next_thresh {
		t.0 = next_thresh.0.to_string();
	}
}

#[derive(Component, Default, Clone)]
struct NextThreshMarker;

fn next() -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
		}
		Children [
			Text("Next Score Check") TextColor(Color::srgb_u8(0x66, 0x66, 0x66)) TextFont { font_size: FontSize::Px(16.0) },
			NextThreshMarker Text("0") TextColor(Color::srgb_u8(0x33, 0x33, 0x33)) TextFont { font_size: FontSize::Px(24.0), weight: FontWeight(700) },
		]
	}
}

#[derive(Component, Default, Clone)]
struct DropCooldownMarker;

fn drop_cooldown() -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
		}
		Children [
			Text("Drop Cooldown") TextColor(Color::srgb_u8(0x66, 0x66, 0x66)) TextFont { font_size: FontSize::Px(16.0) },
			DropCooldownMarker Text("0.00") TextColor(Color::srgb_u8(0x33, 0x33, 0x33)) TextFont { font_size: FontSize::Px(24.0), weight: FontWeight(700) },
		]
	}
}

fn update_cooldown(
	cooldown: Res<FruitDropCooldown>,
	q_text: Query<&mut Text, With<DropCooldownMarker>>,
) {
	if cooldown.just_finished() {
		for mut text in q_text {
			text.0 = String::from("0.00");
		}
		return;
	}

	if cooldown.is_finished() {
		return;
	}

	for mut text in q_text {
		text.0 = format!("{:.2}", cooldown.remaining_secs());
	}
}

fn check_spawned_fruit(
	event: On<SpawnFruit>,
	mut commands: Commands,
	apclient: Res<ApClient>,
	fruit_order: Res<FruitOrder>,
) {
	let Some(last) = fruit_order.order.last() else {
		unreachable!();
	};

	let Some(apclient) = apclient.client() else {
		return;
	};

	let slot_data = apclient.slot_data();

	if event.fruit.kind == last.kind && slot_data.goal.win_by_fruit() {
		commands.trigger(Win);
	}
}

fn on_win(
	_: On<Win>,
	mut commands: Commands,
	mut apclient: ResMut<ApClient>,
	mut retry: Local<f32>,
) {
	let Some(apclient) = apclient.client_mut() else {
		return;
	};

	if let Err(err) = apclient.set_status(ClientStatus::Goal) {
		error!(
			err = err.to_string(),
			"Failed to update client status, retrying in {retry} seconds",
			retry = *retry
		);

		if *retry < 1.0 {
			*retry = 1.0;
		}

		commands.delayed().secs(*retry).trigger(Win);

		if *retry < 32.0 {
			*retry = retry.powf(2.0);
		}
	}
}

fn on_load_savegame(
	event: On<LoadSavedGame>,
	mut next_thresh: ResMut<NextThreshold>,
	apclient: Res<ApClient>,
) {
	let Some(apclient) = apclient.client() else {
		return;
	};

	let Some(next) = get_next_score(
		&apclient.slot_data().scoresanity_thresholds,
		event.score,
		apclient.unchecked_locations(),
	) else {
		return;
	};

	next_thresh.0 = next.value;
}

fn get_next_score(
	scores: &[ScoresanityThreshold],
	score: usize,
	unchecked_locations: impl archipelago_rs::UnsizedIter<archipelago_rs::Location>,
) -> Option<&ScoresanityThreshold> {
	let unchecked_locations: HashSet<i64> = HashSet::from_iter(unchecked_locations.map(|l| l.id()));
	scores
		.iter()
		.find(|t| unchecked_locations.contains(&t.location) && score < t.value)
}

fn monitor_next_fruit_vis(
	can_see: Res<CanSeeNextFruit>,
	q_next_fruits: Query<&mut Visibility, With<NextFruitComponent>>,
) {
	let v = if **can_see {
		Visibility::Inherited
	} else {
		Visibility::Hidden
	};

	for mut vis in q_next_fruits {
		*vis = v;
	}
}

fn smoothie() -> ChatMessage {
	let Some(name) = petname::petname(2, " ") else {
		return ChatMessage::Text("You got a smoothie!".to_string());
	};

	fn random_style(mut node: TextNode) -> TextNode {
		use archipelago_rs::TextColor::{self, *};
		const COLOR_CHOICES: &[TextColor] = &[
			Bold, Underline, Black, Red, Green, Yellow, Blue, Magenta, Cyan, White, BlackBg, RedBg,
			GreenBg, YellowBg, BlueBg, MagentaBg, CyanBg, WhiteBg,
		];
		style_from_apclr(
			&mut node,
			COLOR_CHOICES.choose(&mut rand::rng()).cloned().unwrap(),
		);
		node
	}

	let is_vowel = name
		.chars()
		.next()
		.map(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
		.unwrap_or_default();
	ChatMessage::Rich(RichText::from_iter([
		TextNode::new(format!("You got {} ", if is_vowel { "an" } else { "a" })),
		random_style(TextNode::new(name)),
		TextNode::new(" smoothie!"),
	]))
}

fn on_fruit_combo(event: On<FruitCombo>, mut commands: Commands, mut apclient: ResMut<ApClient>) {
	let Some(apclient) = apclient.client_mut() else {
		return;
	};

	let location = best_fruit_of(event.0);

	if let Err(err) = apclient.mark_checked([location]) {
		commands.write_message(ChatMessage::error(err));
		commands
			.delayed()
			.secs(1.0)
			.trigger(FruitCombo::clone(&event));
	}
}

fn best_fruit_of(fruit: FruitKind) -> i64 {
	match fruit {
		FruitKind::Cherry => 1,
		FruitKind::Strawberry => 2,
		FruitKind::Grapes => 3,
		FruitKind::Dekopon => 4,
		FruitKind::Persimmon => 5,
		FruitKind::Apple => 6,
		FruitKind::Pear => 7,
		FruitKind::Peach => 8,
		FruitKind::Pineapple => 9,
		FruitKind::Melon => 10,
		FruitKind::Watermelon => 11,
	}
}
