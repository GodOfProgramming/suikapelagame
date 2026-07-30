use super::{FruitData, FruitKind};
use crate::game::{
	cmp::Invincible,
	res::{FruitDropDelay, FruitOrder, NextFruit, PlaceholderPosition, StaticAssets},
	scn::{FruitBody, PlaceholderFruit},
};
use bevy::{ecs::system::SystemParam, prelude::*};
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

#[derive(SystemParam)]
pub struct DropPlaceholderRes<'w> {
	next_fruit: ResMut<'w, NextFruit>,
	static_assets: Res<'w, StaticAssets>,
	placeholder_pos: Res<'w, PlaceholderPosition>,
	delay: Res<'w, FruitDropDelay>,
	order: Res<'w, FruitOrder>,
	time: Res<'w, Time>,
}

impl DropPlaceholder {
	pub fn handle(
		_: On<Self>,
		mut commands: Commands,
		placeholder: Single<(Entity, &FruitKind, &Transform), With<PlaceholderFruit>>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
		mut res: DropPlaceholderRes,
	) {
		let (entity, fruit, tf) = *placeholder;
		let elapsed = res.time.elapsed_secs();
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
			AudioPlayer::new(res.static_assets.click.clone()),
			PlaybackSettings::DESPAWN,
		));

		let fruit = res.next_fruit.get(&res.order, &mut rng);

		commands
			.delayed()
			.secs(**res.delay)
			.queue_spawn_scene(bsn! {
				@PlaceholderFruit {
					@fruit: fruit,
					@x: { res.placeholder_pos.x },
				}
			});
	}
}

#[derive(Event)]
pub struct SpawnFruit {
	pub fruit: FruitKind,
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
