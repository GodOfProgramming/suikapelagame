pub mod traps;

use std::time::Duration;

use super::FruitKind;
use crate::{
	GameState,
	game::{
		archipelago::traps::{
			BlindnessTrap, ImpulseTrap, SecrecyTrap, SecrecyTurns, ShuffleTrap, ThanosTrap,
		},
		evt::{DropPlaceholder, PlaceholderDropped, SpawnFruit},
		res::{CanSeeNextFruit, FruitDropDelay, FruitOrder, Score},
		termination::{
			lose::{Lose, LoseHeight},
			win::Win,
		},
	},
	ui::components::{NextFruitComponent, ScoreComponent},
};
use archipelago_rs::{Client, ClientStatus};
use bevy::{ecs::system::SystemParam, prelude::*};
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
			.init_resource::<NextThreshold>()
			.init_resource::<FruitDropCooldown>()
			.add_message::<ApMessage>()
			.add_plugins(TrapPlugin)
			.add_observer(archipelago_error_handler.run_if(resource_exists::<ApClient>))
			.add_observer(ReceivedItem::handle.run_if(resource_exists::<ApClient>))
			.add_observer(ProgressiveHeightItem::handle.run_if(resource_exists::<ApClient>))
			.add_observer(BonusPoints::handle.run_if(resource_exists::<ApClient>))
			.add_observer(CooldownUpgrade::handle.run_if(resource_exists::<ApClient>))
			.add_observer(FruitSizeUpgrade::handle.run_if(resource_exists::<ApClient>))
			.add_observer(NextFruitUnlock::handle.run_if(resource_exists::<ApClient>))
			.add_observer(check_spawned_fruit.run_if(resource_exists::<ApClient>))
			.add_observer(on_win.run_if(resource_exists::<ApClient>))
			.add_observer(on_aplogin)
			.add_observer(on_drop_fruit.run_if(resource_exists::<ApClient>))
			.add_observer(msg::on_chatbox_creation)
			.add_systems(
				Update,
				(
					set_next_scores.run_if(resource_changed::<NextThreshold>),
					tick_cooldown,
				)
					.run_if(resource_exists::<ApClient>.and_then(in_state(GameState::Gameplay))),
			)
			.add_systems(
				FixedUpdate,
				(
					receive_archipelago_events,
					monitor_score.run_if(in_state(GameState::Gameplay)),
				)
					.run_if(resource_exists::<ApClient>),
			);
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct ApClient(Client<SlotData>);

impl ApClient {
	pub fn new(client: Client<SlotData>) -> Self {
		Self(client)
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

fn receive_archipelago_events(
	mut apclient: ResMut<ApClient>,
	mut commands: Commands,
	mut receiving_new_items: Local<bool>,
) {
	for event in apclient.update() {
		match event {
			archipelago_rs::Event::Connected => {
				commands.queue(ApMessage::new("Client connected"));
			}
			archipelago_rs::Event::Updated(_) => {}
			archipelago_rs::Event::Print(print) => {
				commands.queue(ApMessage::from_print(&apclient.this_player().name(), print));
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
			archipelago_rs::Event::Bounce { .. } => {}
			archipelago_rs::Event::DeathLink { source, .. } => {
				commands.queue(ApMessage::new(format!("DeathLink received from {source}")));
				commands.trigger(Lose);
			}
			archipelago_rs::Event::KeyChanged { .. } => {}
		}
	}
}

#[derive(Message, Clone)]
pub enum ApMessage {
	Text(String),
	Rich(String),
}

impl ApMessage {
	pub fn new(msg: impl ToString) -> Self {
		Self::Text(msg.to_string())
	}

	fn from_print(player_name: &str, value: archipelago_rs::Print) -> Self {
		let s = value
			.data()
			.iter()
			.map(|t| match t {
				archipelago_rs::RichText::Player(player) => {
					let tag = if player.name() == player_name {
						msg::tags::SELF
					} else {
						msg::tags::PLAYER
					};
					format!("[{tag}]{}", player.name())
				}
				archipelago_rs::RichText::PlayerName(name) => {
					let tag = if name == player_name {
						msg::tags::SELF
					} else {
						msg::tags::PLAYER
					};
					format!("[{tag}]{name}")
				}
				archipelago_rs::RichText::Item {
					item,
					progression,
					useful,
					trap,
					..
				} => {
					let mut tags = Vec::new();

					if *progression {
						tags.push(msg::tags::PROGRESSION);
					}

					if *useful {
						tags.push(msg::tags::USEFUL);
					}

					if *trap {
						tags.push(msg::tags::TRAP);
					}

					let prefix = if !tags.is_empty() {
						format!("[{}]", tags.join(","))
					} else {
						String::default()
					};

					format!("{prefix}{}", item.name())
				}
				archipelago_rs::RichText::Location { location, .. } => {
					format!("[{}]{}", msg::tags::LOCATION, location.name())
				}
				archipelago_rs::RichText::EntranceName(name) => {
					format!("[{}]{name}", msg::tags::ENTRANCE)
				}
				archipelago_rs::RichText::Color { text, color } => {
					format!("[{}]{text}", msg::apclr_to_string(color.clone()))
				}
				archipelago_rs::RichText::Text(t) => {
					format!("[]{t}")
				}
			})
			.collect();
		Self::Rich(s)
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

impl NextFruitUnlock {
	fn handle(
		_: On<Self>,
		q_next: Query<&mut Visibility, With<NextFruitComponent>>,
		secrecy: Res<SecrecyTurns>,
		mut next_visible: ResMut<CanSeeNextFruit>,
	) {
		**next_visible = true;

		if secrecy.inactive() {
			for mut vis in q_next {
				*vis = Visibility::Inherited;
			}
		}
	}
}

#[derive(Event)]
pub struct ApLoginComplete;

#[derive(SystemParam)]
pub struct LoginRes<'w> {
	apclient: Res<'w, ApClient>,
	fruit_order: ResMut<'w, FruitOrder>,
	lose_height: ResMut<'w, LoseHeight>,
	drop_delay: ResMut<'w, FruitDropDelay>,
	can_see_next: ResMut<'w, CanSeeNextFruit>,
}

fn on_aplogin(
	_: On<ApLoginComplete>,
	mut commands: Commands,
	q_scores: Query<Entity, With<ScoreComponent>>,
	mut res: LoginRes,
) {
	for entity in q_scores {
		commands.queue_spawn_scene_list(bsn_list! [
			ChildOf(entity) next(),
			ChildOf(entity) drop_cooldown(),
		]);
	}

	let slot_data = res.apclient.slot_data();
	if let Some(shuffled) = slot_data.shuffle_fruit {
		res
			.fruit_order
			.set_order(shuffled, slot_data.also_shuffle_sizes);
	}

	res.lose_height.max_upgrades = slot_data.height_upgrade_count;

	res.drop_delay.additional_cooldown = slot_data.starting_cooldown as f32;
	res.drop_delay.max_upgrades = slot_data.cooldown_upgrade_count;

	res.fruit_order.limit = slot_data.max_fruit_size;
	**res.can_see_next = !slot_data.next_needs_unlock;
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
	let v = next_thresh.0.to_string();
	for mut t in q_next_thresh {
		t.0 = v.clone();
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

#[derive(Resource, Reflect, Deref, DerefMut)]
struct FruitDropCooldown(Timer);

impl FromWorld for FruitDropCooldown {
	fn from_world(world: &mut World) -> Self {
		let delay = world.resource::<FruitDropDelay>();
		Self(Timer::new(
			Duration::from_secs_f32(delay.delay()),
			TimerMode::Once,
		))
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

fn on_drop_fruit(
	_: On<PlaceholderDropped>,
	mut cooldown: ResMut<FruitDropCooldown>,
	delay: Res<FruitDropDelay>,
) {
	cooldown.reset();
	cooldown.set_duration(Duration::from_secs_f32(delay.delay()));
}

fn tick_cooldown(
	q_text: Query<&mut Text, With<DropCooldownMarker>>,
	mut cooldown: ResMut<FruitDropCooldown>,
	time: Res<Time>,
	delay: Res<FruitDropDelay>,
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

	cooldown.tick(time.delta());

	for mut text in q_text {
		text.0 = format!("{:.2}", delay.delay() - cooldown.elapsed_secs());
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

mod msg {
	use archipelago_rs::TextColor as ApTextColor;
	use bevy::prelude::*;
	use bevy_simple_rich_text::StyleTag;

	use crate::ui::components::chat::ChatComponent;

	pub mod tags {
		pub const PROGRESSION: &str = "progression";
		pub const USEFUL: &str = "useful";
		pub const FILLER: &str = "filler";
		pub const TRAP: &str = "trap";
		pub const LOCATION: &str = "location";
		pub const ENTRANCE: &str = "entrance";
		pub const SELF: &str = "self";
		pub const PLAYER: &str = "player";
	}

	pub fn apclr_to_string(color: ApTextColor) -> &'static str {
		match color {
			archipelago_rs::TextColor::Bold => "bold",
			archipelago_rs::TextColor::Underline => "underline",
			archipelago_rs::TextColor::Black => "black",
			archipelago_rs::TextColor::Red => "red",
			archipelago_rs::TextColor::Green => "green",
			archipelago_rs::TextColor::Yellow => "yellow",
			archipelago_rs::TextColor::Blue => "blue",
			archipelago_rs::TextColor::Magenta => "magenta",
			archipelago_rs::TextColor::Cyan => "cyan",
			archipelago_rs::TextColor::White => "white",
			archipelago_rs::TextColor::BlackBg => "black_bg",
			archipelago_rs::TextColor::RedBg => "red_bg",
			archipelago_rs::TextColor::GreenBg => "green_bg",
			archipelago_rs::TextColor::YellowBg => "yellow_bg",
			archipelago_rs::TextColor::BlueBg => "blue_bg",
			archipelago_rs::TextColor::MagentaBg => "magenta_bg",
			archipelago_rs::TextColor::CyanBg => "cyan_bg",
			archipelago_rs::TextColor::WhiteBg => "white_bg",
		}
	}

	pub fn on_chatbox_creation(event: On<Add, ChatComponent>, mut commands: Commands) {
		commands.spawn_batch([
			(
				ChildOf(event.event_target()),
				ApTextStyle::from(ApTextStyleConfig {
					tag: tags::PROGRESSION,
					color: Color::srgb_u8(168, 147, 228),
					weight: FontWeight::BOLD,
				}),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::USEFUL,
					color: Color::srgb_u8(98, 122, 198),
					weight: FontWeight::BOLD,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::FILLER,
					color: Color::srgb_u8(6, 217, 217),
					weight: FontWeight::BOLD,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::TRAP,
					color: Color::srgb_u8(211, 113, 102),
					weight: FontWeight::BOLD,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::LOCATION,
					color: Color::srgb_u8(0, 255, 127),
					weight: FontWeight::NORMAL,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::ENTRANCE,
					color: Color::srgb(0.17, 0.01, 0.46),
					weight: FontWeight::NORMAL,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::SELF,
					color: Color::srgb_u8(238, 0, 238),
					weight: FontWeight::BOLD,
				}
				.into(),
			),
			(
				ChildOf(event.event_target()),
				ApTextStyleConfig {
					tag: tags::PLAYER,
					color: Color::srgb_u8(150, 150, 110),
					weight: FontWeight::BOLD,
				}
				.into(),
			),
		]);

		commands.spawn((
			ChildOf(event.event_target()),
			StyleTag::new("bold"),
			TextFont::from_font_weight(FontWeight::BOLD),
		));

		commands.spawn((
			ChildOf(event.event_target()),
			StyleTag::new("underline"),
			Underline,
		));

		let clr = |txt_color: ApTextColor, color: Color| -> (ChildOf, StyleTag, TextColor) {
			(
				ChildOf(event.event_target()),
				StyleTag::new(apclr_to_string(txt_color)),
				TextColor(color),
			)
		};

		let clr_bg = |txt_color: ApTextColor,
		              color: Color,
		              bg: Color|
		 -> (ChildOf, StyleTag, TextColor, BackgroundColor) {
			(
				ChildOf(event.event_target()),
				StyleTag::new(apclr_to_string(txt_color)),
				TextColor(color),
				BackgroundColor(bg),
			)
		};

		let bg = |txt_color: ApTextColor, color: Color| -> (ChildOf, StyleTag, BackgroundColor) {
			(
				ChildOf(event.event_target()),
				StyleTag::new(apclr_to_string(txt_color)),
				BackgroundColor(color),
			)
		};

		commands.spawn_batch([
			clr(ApTextColor::Black, Color::BLACK),
			clr(ApTextColor::Red, Color::srgb_u8(255, 64, 63)),
			clr(ApTextColor::Blue, Color::srgb_u8(63, 64, 255)),
			clr(ApTextColor::Magenta, Color::srgb_u8(255, 127, 255)),
		]);

		commands.spawn_batch([
			clr_bg(
				ApTextColor::Green,
				Color::srgb_u8(0, 255, 127),
				Color::srgba(0.0, 0.0, 0.0, 0.25),
			),
			clr_bg(
				ApTextColor::Yellow,
				Color::srgb_u8(255, 255, 127),
				Color::srgba(0.0, 0.0, 0.0, 0.5),
			),
			clr_bg(
				ApTextColor::Cyan,
				Color::srgb_u8(127, 255, 255),
				Color::srgba(0.0, 0.0, 0.0, 0.5),
			),
			clr_bg(
				ApTextColor::White,
				Color::WHITE,
				Color::srgba(0.0, 0.0, 0.0, 0.5),
			),
			clr_bg(
				ApTextColor::BlackBg,
				Color::srgb_u8(0xcc, 0xcc, 0xcc),
				Color::BLACK,
			),
			clr_bg(
				ApTextColor::BlueBg,
				Color::srgb_u8(0xcc, 0xcc, 0xcc),
				Color::srgb_u8(63, 64, 255),
			),
		]);

		commands.spawn_batch([
			bg(ApTextColor::RedBg, Color::srgb_u8(255, 64, 63)),
			bg(ApTextColor::GreenBg, Color::srgb_u8(0, 255, 127)),
			bg(ApTextColor::YellowBg, Color::srgb_u8(255, 255, 127)),
			bg(ApTextColor::MagentaBg, Color::srgb_u8(255, 127, 255)),
			bg(ApTextColor::CyanBg, Color::srgb_u8(127, 255, 255)),
			bg(ApTextColor::WhiteBg, Color::WHITE),
		]);
	}

	#[derive(Bundle)]
	struct ApTextStyle {
		tag: StyleTag,
		color: TextColor,
		font: TextFont,
	}

	struct ApTextStyleConfig<T, C, W>
	where
		T: ToString,
		C: Into<Color>,
		W: Into<FontWeight>,
	{
		tag: T,
		color: C,
		weight: W,
	}

	impl<T, C, W> From<ApTextStyleConfig<T, C, W>> for ApTextStyle
	where
		T: ToString,
		C: Into<Color>,
		W: Into<FontWeight>,
	{
		fn from(value: ApTextStyleConfig<T, C, W>) -> Self {
			Self {
				tag: StyleTag(value.tag.to_string()),
				color: TextColor(value.color.into()),
				font: TextFont::from_font_weight(value.weight.into()),
			}
		}
	}
}
