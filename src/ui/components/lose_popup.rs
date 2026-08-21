use bevy::prelude::*;

use crate::{GameState, ui::MiddleUi, util::despawn_query};

pub struct LosePopupPlugin;

impl Plugin for LosePopupPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				OnEnter(GameState::LoseGame),
				super::spawn_state_popup::<With<MiddleUi>>(
					"Game Over!",
					Color::srgb_u8(0xff, 0x2b, 0x00),
					Color::srgb_u8(0xff, 0x54, 0x00),
				),
			)
			.add_systems(
				OnExit(GameState::LoseGame),
				despawn_query::<With<super::StatePopup>>,
			);
	}
}
