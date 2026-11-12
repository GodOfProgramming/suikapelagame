use bevy::prelude::*;

use crate::{
	GameState,
	game::{FruitKind, archipelago::ApClient, res::FruitOrder},
};

pub struct WinPlugin;

impl Plugin for WinPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(Win::handle).add_observer(TryWin::handle);
	}
}

#[derive(Event)]
pub enum TryWin {
	ViaFruit(FruitKind),
	ViaScore(usize),
}

impl TryWin {
	fn handle(
		event: On<Self>,
		mut commands: Commands,
		apclient: Res<ApClient>,
		fruit_order: Res<FruitOrder>,
		mut score_goal_reached: Local<bool>,
		mut fruit_goal_reached: Local<bool>,
	) {
		let Some(apclient) = apclient.client() else {
			return;
		};

		let Some(last) = fruit_order.order.last() else {
			unreachable!();
		};

		let slot_data = apclient.slot_data();

		match &*event {
			TryWin::ViaFruit(fruit_kind) => {
				if !*fruit_goal_reached && *fruit_kind == last.kind {
					match slot_data.goal {
						crate::game::archipelago::Goal::LargestFruit => {
							commands.trigger(Win);
						}
						crate::game::archipelago::Goal::Both => {
							*fruit_goal_reached = true;
							if *score_goal_reached {
								commands.trigger(Win);
							}
						}
						crate::game::archipelago::Goal::Score => {}
					}
				}
			}
			TryWin::ViaScore(index) => {
				if *index >= slot_data.scoresanity_thresholds.len() {
					match slot_data.goal {
						crate::game::archipelago::Goal::Score => {
							commands.trigger(Win);
						}
						crate::game::archipelago::Goal::Both => {
							*score_goal_reached = true;
							if *fruit_goal_reached {
								commands.trigger(Win);
							}
						}
						crate::game::archipelago::Goal::LargestFruit => {}
					}
				}
			}
		}
	}
}

#[derive(Event)]
pub struct Win;

impl Win {
	fn handle(_: On<Self>, mut game_state: ResMut<NextState<GameState>>) {
		game_state.set(GameState::WinGame);
	}
}
