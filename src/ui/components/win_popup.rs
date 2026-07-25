use bevy::prelude::*;

use crate::{GameState, ui::MiddleUi, util::despawn_query};

pub struct WinPopupPlugin;

impl Plugin for WinPopupPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(
				OnEnter(GameState::WinGame),
				super::spawn_state_popup::<With<MiddleUi>>(
					"You Win",
					Color::srgb_u8(0x38, 0x59, 0x00),
					Color::srgb_u8(0x00, 0x55, 0x00),
				),
			)
			.add_systems(
				OnExit(GameState::WinGame),
				despawn_query::<With<super::StatePopup>>,
			);
	}
}
