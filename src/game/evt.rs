use super::{Fruit, FruitData};
use crate::game::{
	cmp::Invincible,
	res::{FruitDropDelay, NextFruit, PlaceholderPosition, StaticAssets},
	scn::{FruitBody, PlaceholderFruit},
};
use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;

#[derive(Event, Clone, Deref)]
pub struct MovePlaceholder(pub Vec2);

impl MovePlaceholder {
	pub fn handle(event: On<Self>, mut placeholder: ResMut<PlaceholderPosition>) {
		placeholder.0 = event.0.xy();
	}
}

#[derive(Event, Clone)]
pub struct DropPlaceholder;

impl DropPlaceholder {
	pub fn handle(
		_: On<Self>,
		mut commands: Commands,
		placeholder: Single<(Entity, &Fruit, &Transform), With<PlaceholderFruit>>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
		mut next_fruit: ResMut<NextFruit>,
		static_assets: Res<StaticAssets>,
		placeholder_pos: Res<PlaceholderPosition>,
		delay: Res<FruitDropDelay>,
		time: Res<Time>,
	) {
		let (entity, fruit, tf) = *placeholder;
		let elapsed = time.elapsed_secs();
		commands
			.entity(entity)
			.remove::<PlaceholderFruit>()
			.apply_scene(bsn! {
				@FruitBody {
					@fruit: { *fruit },
					@position: { tf.translation.xy() }
				}
				Invincible::new(elapsed)
			});

		commands.spawn((
			AudioPlayer::new(static_assets.click.clone()),
			PlaybackSettings::DESPAWN,
		));

		let fruit = next_fruit.next(&mut rng);

		commands.delayed().secs(**delay).queue_spawn_scene(bsn! {
			@PlaceholderFruit {
				@fruit: fruit,
				@x: { placeholder_pos.x },
			}
		});
	}
}

#[derive(Event)]
pub struct SpawnFruit {
	pub fruit: Fruit,
	pub data: FruitData,
	pub position: Vec2,
	pub invincibility: f32,
}

impl SpawnFruit {
	pub fn handle(event: On<Self>, mut commands: Commands) {
		let invincibility = event.invincibility;
		commands.spawn_scene(bsn! {
			@FruitBody {
				@fruit: {event.fruit},
				@position: { event.position },
				@custom: { Some(event.data) }
			}
			Invincible::new(invincibility)
		});
	}
}
