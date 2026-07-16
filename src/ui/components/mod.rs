pub mod fruit_circle;
pub mod login_form;
pub mod next_fruit;
pub mod score;

use crate::util::math;
use bevy::{
	color::palettes::{css::DARK_GREY, tailwind::SLATE_300},
	prelude::*,
	text::{EditableText, TextCursorStyle},
};

pub use {
	fruit_circle::FruitCircleComponent, login_form::LoginFormComponent,
	next_fruit::NextFruitComponent, score::ScoreComponent,
};

#[derive(SceneComponent, Default, Clone)]
#[scene(AppEditableTextProps)]
pub struct AppEditableText;

#[derive(Default)]
pub struct AppEditableTextProps {
	name: String,
	placeholder_text: String,
}

impl AppEditableText {
	fn scene(props: AppEditableTextProps) -> impl Scene {
		let name = Name::new(props.name);

		let mut text = EditableText::new(props.placeholder_text);
		text.visible_width = Some(10.);
		text.allow_newlines = false;

		bsn! {
			Node {
				border: { px(2).all() }
			}
			BorderColor::from(Color::from(SLATE_300))
			template_value(name)
			template_value(text)
			TextLayout::no_wrap()
			TextCursorStyle::default()
			BackgroundColor(DARK_GREY)
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(FruitImageProps)]
pub struct FruitImage;

#[derive(Default)]
pub struct FruitImageProps {
	/// Only as a percent
	pub width: f32,
	pub image: String,
	pub degrees: f32,
}

impl FruitImage {
	fn scene(props: FruitImageProps) -> impl Scene {
		let offset = 50.0 - props.width / 2.0;

		let (bottom, left) = math::position_by_degrees(props.degrees, offset);

		bsn! {
			Node {
				width: percent(props.width),
				aspect_ratio: { Some(1.0) },
				position_type: PositionType::Absolute,
				left: { percent(left) },
				bottom: { percent(bottom) },
			}
			ImageNode {
				image: { props.image },
			}
		}
	}
}

fn column(children: impl SceneList) -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
		}
		Children [
			{children}
		]
	}
}
