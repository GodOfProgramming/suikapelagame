pub mod styles;

use archipelago_rs::TextColor as ApTextColor;
use bevy::prelude::*;
use std::{borrow::Cow, sync::Arc};

#[derive(Message, Clone)]
pub enum ApMessage {
	Text(String),
	Rich(RichText),
}

impl ApMessage {
	pub fn new(msg: impl ToString) -> Self {
		Self::Text(msg.to_string())
	}

	pub fn error(msg: impl std::fmt::Display) -> Self {
		error!("error: {msg}");
		Self::Rich(RichText::from_iter([
			TextNode::new("error: "),
			TextNode::new(msg.to_string()).with_style(styles::RED),
		]))
	}

	pub fn from_print(player_name: &str, value: archipelago_rs::Print) -> Self {
		let text = value
			.data()
			.iter()
			.map(|t| match t {
				archipelago_rs::RichText::Player(player) => {
					let mut node = TextNode::new(player.name().to_string());
					if player.name() == player_name {
						node.style(styles::SELF);
					} else {
						node.style(styles::PLAYER);
					};
					node
				}
				archipelago_rs::RichText::PlayerName(name) => {
					let mut node = TextNode::new(name.to_string());
					if name == player_name {
						node.style(styles::SELF);
					} else {
						node.style(styles::PLAYER);
					};
					node
				}
				archipelago_rs::RichText::Item {
					item,
					progression,
					useful,
					trap,
					..
				} => {
					let mut node = TextNode::new(item.name().to_string());

					if *progression {
						node.style(styles::PROGRESSION);
					}

					if *useful {
						node.style(styles::USEFUL);
					}

					if *trap {
						node.style(styles::TRAP);
					}

					node
				}
				archipelago_rs::RichText::Location { location, .. } => {
					TextNode::new(location.name().to_string()).with_style(styles::LOCATION)
				}
				archipelago_rs::RichText::EntranceName(name) => {
					TextNode::new(name.to_string()).with_style(styles::ENTRANCE)
				}
				archipelago_rs::RichText::Color { text, color } => {
					let mut node = TextNode::new(text.to_string());
					style_from_apclr(&mut node, color.clone());
					node
				}
				archipelago_rs::RichText::Text(t) => TextNode::new(t.clone()),
			})
			.collect::<RichText>();
		Self::Rich(text)
	}
}

impl Command for ApMessage {
	type Out = ();
	fn apply(self, world: &mut World) -> Self::Out {
		world.write_message(self);
	}
}

pub fn style_from_apclr(node: &mut TextNode, color: ApTextColor) {
	match color {
		archipelago_rs::TextColor::Bold => node.style(styles::BOLD),
		archipelago_rs::TextColor::Underline => node.style(styles::UNDERLINE),
		archipelago_rs::TextColor::Black => node.style(styles::BLACK),
		archipelago_rs::TextColor::Red => node.style(styles::RED),
		archipelago_rs::TextColor::Green => node.style(styles::GREEN),
		archipelago_rs::TextColor::Yellow => node.style(styles::YELLOW),
		archipelago_rs::TextColor::Blue => node.style(styles::BLUE),
		archipelago_rs::TextColor::Magenta => node.style(styles::MAGENTA),
		archipelago_rs::TextColor::Cyan => node.style(styles::CYAN),
		archipelago_rs::TextColor::White => node.style(styles::WHITE),
		archipelago_rs::TextColor::BlackBg => node.style(styles::BLACK_BG),
		archipelago_rs::TextColor::RedBg => node.style(styles::RED_BG),
		archipelago_rs::TextColor::GreenBg => node.style(styles::GREEN_BG),
		archipelago_rs::TextColor::YellowBg => node.style(styles::YELLOW_BG),
		archipelago_rs::TextColor::BlueBg => node.style(styles::BLUE_BG),
		archipelago_rs::TextColor::MagentaBg => node.style(styles::MAGENTA_BG),
		archipelago_rs::TextColor::CyanBg => node.style(styles::CYAN_BG),
		archipelago_rs::TextColor::WhiteBg => node.style(styles::WHITE_BG),
	};
}

#[derive(Clone)]
pub struct RichText(Vec<TextNode>);

impl RichText {
	pub fn iter(&self) -> impl Iterator<Item = &TextNode> {
		self.0.iter()
	}
}

impl FromIterator<TextNode> for RichText {
	fn from_iter<T: IntoIterator<Item = TextNode>>(iter: T) -> Self {
		Self(Vec::from_iter(iter))
	}
}

pub trait Style: Send + Sync {
	fn apply(&self, cmds: &mut EntityCommands);
}

#[derive(Clone)]
pub struct TextNode {
	text: Cow<'static, str>,
	styles: Vec<Arc<dyn Style>>,
}

impl TextNode {
	pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
		Self {
			text: text.into(),
			styles: Vec::default(),
		}
	}

	pub fn text(&self) -> &str {
		self.text.as_ref()
	}

	pub fn style(&mut self, style: impl Style + 'static) -> &mut Self {
		self.styles.push(Arc::new(style));
		self
	}

	pub fn with_style(mut self, style: impl Style + 'static) -> Self {
		self.style(style);
		self
	}

	pub fn styles(&self) -> impl Iterator<Item = &Arc<dyn Style>> {
		self.styles.iter()
	}
}
