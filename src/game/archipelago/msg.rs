use archipelago_rs::TextColor as ApTextColor;
use bevy::prelude::*;
use bevy_simple_rich_text::StyleTag;

use crate::ui::components::chat::ChatComponent;

#[derive(Message, Clone)]
pub enum ApMessage {
	Text(String),
	Rich(String),
}

impl ApMessage {
	pub fn new(msg: impl ToString) -> Self {
		Self::Text(msg.to_string())
	}

	pub fn error(msg: impl std::fmt::Display) -> Self {
		error!("error: {msg}");
		Self::Rich(format!("error: [{color}]{msg}", color = tags::RED))
	}

	pub fn from_print(player_name: &str, value: archipelago_rs::Print) -> Self {
		let s = value
			.data()
			.iter()
			.map(|t| match t {
				archipelago_rs::RichText::Player(player) => {
					let tag = if player.name() == player_name {
						tags::SELF
					} else {
						tags::PLAYER
					};
					format!("[{tag}]{}", player.name())
				}
				archipelago_rs::RichText::PlayerName(name) => {
					let tag = if name == player_name {
						tags::SELF
					} else {
						tags::PLAYER
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
						tags.push(tags::PROGRESSION);
					}

					if *useful {
						tags.push(tags::USEFUL);
					}

					if *trap {
						tags.push(tags::TRAP);
					}

					let prefix = if !tags.is_empty() {
						format!("[{}]", tags.join(","))
					} else {
						String::default()
					};

					format!("{prefix}{}", item.name())
				}
				archipelago_rs::RichText::Location { location, .. } => {
					format!("[{}]{}", tags::LOCATION, location.name())
				}
				archipelago_rs::RichText::EntranceName(name) => {
					format!("[{}]{name}", tags::ENTRANCE)
				}
				archipelago_rs::RichText::Color { text, color } => {
					format!("[{}]{text}", apclr_to_string(color.clone()))
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

pub mod tags {
	pub const PROGRESSION: &str = "progression";
	pub const USEFUL: &str = "useful";
	pub const FILLER: &str = "filler";
	pub const TRAP: &str = "trap";
	pub const LOCATION: &str = "location";
	pub const ENTRANCE: &str = "entrance";
	pub const SELF: &str = "self";
	pub const PLAYER: &str = "player";
	pub const BOLD: &str = "bold";
	pub const UNDERLINE: &str = "underline";
	pub const BLACK: &str = "black";
	pub const RED: &str = "red";
	pub const GREEN: &str = "green";
	pub const YELLOW: &str = "yellow";
	pub const BLUE: &str = "blue";
	pub const MAGENTA: &str = "magenta";
	pub const CYAN: &str = "cyan";
	pub const WHITE: &str = "white";
	pub const BLACK_BG: &str = "black_bg";
	pub const RED_BG: &str = "red_bg";
	pub const GREEN_BG: &str = "green_bg";
	pub const YELLOW_BG: &str = "yellow_bg";
	pub const BLUE_BG: &str = "blue_bg";
	pub const MAGENTA_BG: &str = "magenta_bg";
	pub const CYAN_BG: &str = "cyan_bg";
	pub const WHITE_BG: &str = "white_bg";
}

pub fn apclr_to_string(color: ApTextColor) -> &'static str {
	match color {
		archipelago_rs::TextColor::Bold => tags::BOLD,
		archipelago_rs::TextColor::Underline => tags::UNDERLINE,
		archipelago_rs::TextColor::Black => tags::BLACK,
		archipelago_rs::TextColor::Red => tags::RED,
		archipelago_rs::TextColor::Green => tags::GREEN,
		archipelago_rs::TextColor::Yellow => tags::YELLOW,
		archipelago_rs::TextColor::Blue => tags::BLUE,
		archipelago_rs::TextColor::Magenta => tags::MAGENTA,
		archipelago_rs::TextColor::Cyan => tags::CYAN,
		archipelago_rs::TextColor::White => tags::WHITE,
		archipelago_rs::TextColor::BlackBg => tags::BLACK_BG,
		archipelago_rs::TextColor::RedBg => tags::RED_BG,
		archipelago_rs::TextColor::GreenBg => tags::GREEN_BG,
		archipelago_rs::TextColor::YellowBg => tags::YELLOW_BG,
		archipelago_rs::TextColor::BlueBg => tags::BLUE_BG,
		archipelago_rs::TextColor::MagentaBg => tags::MAGENTA_BG,
		archipelago_rs::TextColor::CyanBg => tags::CYAN_BG,
		archipelago_rs::TextColor::WhiteBg => tags::WHITE_BG,
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
