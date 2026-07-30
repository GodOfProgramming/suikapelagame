use super::FruitKind;
use crate::game::{
	Fruit,
	cmp::Invincible,
	res::{FruitOrder, NextFruit, PlaceholderPosition, Score},
	scn::{FruitBody, PlaceholderFruit},
};
use avian2d::prelude::*;
use bevy::{
	asset::AssetPath,
	ecs::{query::QueryData, system::SystemParam},
	prelude::*,
};
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use serde::{Deserialize, Serialize};

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
	fn build(&self, app: &mut App) {
		app
			.add_observer(SaveGame::handle)
			.add_observer(LoadSaveFile::handle)
			.add_observer(LoadSavedGame::handle);
	}
}

#[derive(QueryData)]
struct FruitDataToSave {
	fruit: &'static Fruit,
	transform: &'static Transform,
	linear_velocity: &'static LinearVelocity,
	angular_velocity: &'static AngularVelocity,
	sprite: &'static Sprite,
	invincibility: Option<&'static Invincible>,
}

#[derive(SystemParam)]
struct SaveParams<'w, 's> {
	score: Res<'w, Score>,
	placeholder: Single<'w, 's, &'static Fruit, With<PlaceholderFruit>>,
	next: Res<'w, NextFruit>,
	rng: Single<'w, 's, &'static WyRand, With<GlobalRng>>,
	fruits: Query<'w, 's, FruitDataToSave, Without<PlaceholderFruit>>,
}

#[derive(Event)]
pub struct SaveGame;

impl SaveGame {
	fn handle(_: On<Self>, mut commands: Commands, params: SaveParams) -> Result {
		let fruits = params
			.fruits
			.iter()
			.map(|f| FruitSaveData {
				kind: f.fruit.kind,
				value: f.fruit.data.value,
				transform: *f.transform,
				linear_velocity: *f.linear_velocity,
				angular_velocity: *f.angular_velocity,
				sprite: f.sprite.image.path().cloned().unwrap_or_default(),
				invincibility: f.invincibility.map(|i| **i),
			})
			.collect();

		let savegame = SaveGameData {
			score: **params.score,
			placeholder: params.placeholder.kind,
			next: params.next.kind,
			random_state: params.rng.clone(),
			fruits,
		};

		let out = postcard::to_stdvec(&savegame)?;

		commands.spawn_task(async || {
			let Some(this_dir) = std::env::current_exe()
				.ok()
				.and_then(|f| f.parent().map(|p| p.to_path_buf()))
			else {
				return Ok(());
			};

			if let Err(err) = smol::fs::write(this_dir.join("save.bin"), out).await {
				error!("Failed to save game: {err}");
				return Ok(());
			}

			Ok(())
		});

		Ok(())
	}
}

#[derive(Event)]
pub struct LoadSaveFile;

impl LoadSaveFile {
	fn handle(_: On<Self>, mut commands: Commands, q_fruits: Query<Entity, With<Fruit>>) {
		for fruit in q_fruits {
			commands.entity(fruit).despawn();
		}

		commands.spawn_task(async || {
			let Some(this_dir) = std::env::current_exe()
				.ok()
				.and_then(|f| f.parent().map(|p| p.to_path_buf()))
			else {
				return Ok(());
			};

			let bytes = match smol::fs::read(this_dir.join("save.bin")).await {
				Ok(bytes) => bytes,
				Err(err) => {
					error!("Failed to load game: {err}");
					return Ok(());
				}
			};

			let data = match postcard::from_bytes(&bytes) {
				Ok(data) => data,
				Err(err) => {
					error!("Failed to load game: {err}");
					return Ok(());
				}
			};

			AsyncWorld.trigger_event(LoadSavedGame(data));

			Ok(())
		});
	}
}

#[derive(Serialize, Deserialize)]
struct SaveGameData {
	score: usize,
	placeholder: FruitKind,
	next: FruitKind,
	random_state: WyRand,
	fruits: Vec<FruitSaveData>,
}

#[derive(Serialize, Deserialize)]
struct FruitSaveData {
	kind: FruitKind,
	value: usize,
	transform: Transform,
	linear_velocity: LinearVelocity,
	angular_velocity: AngularVelocity,
	invincibility: Option<f32>,
	sprite: AssetPath<'static>,
}

#[derive(SystemParam)]
struct LoadRes<'w> {
	score: ResMut<'w, Score>,
	placeholder_pos: Res<'w, PlaceholderPosition>,
	next: ResMut<'w, NextFruit>,
	order: Res<'w, FruitOrder>,
}

#[derive(Event, Deref)]
struct LoadSavedGame(SaveGameData);

impl LoadSavedGame {
	fn handle(
		event: On<Self>,
		mut commands: Commands,
		mut rng: Single<&'static mut WyRand, With<GlobalRng>>,
		mut res: LoadRes,
	) {
		**rng = event.random_state.clone();

		**res.score = event.score;
		**res.next = res
			.order
			.get(event.next)
			.cloned()
			.expect("Unable to find fruit in order");

		commands.queue_spawn_scene(bsn! {
			@PlaceholderFruit {
				@fruit: { event.placeholder },
				@x: { res.placeholder_pos.x },
			}
		});

		for fruit in &event.fruits {
			commands.queue_spawn_scene(bsn! {
				@FruitBody {
					@fruit: { res.order.get(fruit.kind).cloned().expect("Unable to find fruit in order") },
				}
				CollisionEventsEnabled
				template_value(fruit.transform)
				template_value(fruit.linear_velocity)
				template_value(fruit.angular_velocity)
				template_value(RigidBody::Dynamic)
				{ fruit.invincibility.map(|i| template_value(Invincible::new(i))) }
			});
		}
	}
}
