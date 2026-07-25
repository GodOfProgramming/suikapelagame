use crate::{
	component_name,
	game::{LoadGame, SaveGame},
	ui::components::AppButton,
};
use bevy::prelude::*;

#[derive(SceneComponent, Default, Clone)]
pub struct SaveLoadComponent;

impl SaveLoadComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(SaveLoadComponent) }
			Node {
				padding: UiRect::all(px(16)),
				border_radius: BorderRadius::all(px(16))
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
			}
			BackgroundColor({ Color::srgb_u8(0xff, 0xea, 0xce) })
			Children [
				@SaveButton,
				@LoadButton,
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct SaveButton;

impl SaveButton {
	fn scene() -> impl Scene {
		bsn! {
			@AppButton
			Node { justify_content: JustifyContent::Center }
			Children [ Text::new("Save") TextColor::BLACK ]
			on(|_: On<Pointer<Click>>, mut commands: Commands| {
				commands.trigger(SaveGame);
			})
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct LoadButton;

impl LoadButton {
	fn scene() -> impl Scene {
		bsn! {
			@AppButton
			Node { justify_content: JustifyContent::Center }
			Children [ Text::new("Load") TextColor::BLACK ]
			on(|_: On<Pointer<Click>>, mut commands: Commands| {
				commands.trigger(LoadGame);
			})
		}
	}
}
