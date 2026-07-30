use archipelago_rs::DeathLinkOptions;
use bevy::prelude::*;

use crate::{
	GameState,
	game::{Fruit, FruitKind, Invincible, WORLD_DIM, archipelago::ApClient, scn::PlaceholderFruit},
};

pub struct LosePlugin;

impl Plugin for LosePlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<LoseHeight>()
			.add_observer(Lose::handle)
			.add_systems(
				FixedUpdate,
				check_if_lose
					.after(TransformSystems::Propagate)
					.run_if(in_state(GameState::Gameplay)),
			);
	}
}

#[derive(Event)]
struct Lose;

impl Lose {
	fn handle(
		_: On<Self>,
		mut game_state: ResMut<NextState<GameState>>,
		apclient: Option<ResMut<ApClient>>,
	) -> Result {
		game_state.set(GameState::LoseGame);

		let Some(mut apclient) = apclient else {
			return Ok(());
		};

		let player = apclient.this_player();
		let cause = format!("{} topped out!", player.name());
		apclient.death_link(DeathLinkOptions::new().cause(cause))?;
		Ok(())
	}
}

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct LoseHeight(f32);

impl Default for LoseHeight {
	fn default() -> Self {
		Self(WORLD_DIM.y * 1.5)
	}
}

fn check_if_lose(
	mut commands: Commands,
	q_tf: Query<(&Transform, &Fruit), (Without<Invincible>, Without<PlaceholderFruit>)>,
	end_height: Res<LoseHeight>,
) {
	for (tf, fruit) in q_tf {
		let y = tf.translation.y + fruit.data.radius * 2.0;
		if y >= **end_height {
			commands.trigger(Lose);
			return;
		}
	}
}
