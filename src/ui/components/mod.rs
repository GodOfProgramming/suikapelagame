pub mod fruit_circle;
pub mod login_form;
pub mod next_fruit;
pub mod score;

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
