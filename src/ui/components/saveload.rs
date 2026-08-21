use crate::{
	component_name,
	game::{
		evt::NewGame,
		persistence::{LoadSaveFile, SaveGame},
	},
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
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
			}
			Children [
				@SaveButton,
				@LoadButton,
				@ResetButton,
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
				commands.trigger(LoadSaveFile);
			})
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct ResetButton;

impl ResetButton {
	fn scene() -> impl Scene {
		bsn! {
			@AppButton
			Node { justify_content: JustifyContent::Center }
			Children [ Text::new("Reset") TextColor::BLACK ]
			on(|_: On<Pointer<Click>>, mut commands: Commands| {
				commands.trigger(NewGame);
			})
		}
	}
}
