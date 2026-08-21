use std::time::Duration;

use crate::game::{
	Fruit, FruitKind,
	cmp::Invincible,
	res::{
		FruitDropCooldown, FruitDropDelay, FruitOrder, NextFruit, PlaceholderPosition, Score,
		StaticAssets,
	},
	scn::{FruitBody, PlaceholderFruit},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use rand::Rng;

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
		static_assets: Res<StaticAssets>,
		time: Res<Time>,
	) {
		let (entity, fruit, tf) = *placeholder;
		let elapsed = time.elapsed_secs();
		commands
			.entity(entity)
			.remove::<PlaceholderFruit>()
			.apply_scene(bsn! {
				@FruitBody {
					@fruit: { fruit.clone() },
					@position: { tf.translation.xy() }
				}
				Invincible::new(elapsed)
			});

		commands.spawn((
			AudioPlayer::new(static_assets.click.clone()),
			PlaybackSettings::DESPAWN,
		));

		commands.trigger(PlaceholderDropped);
	}
}

#[derive(Event)]
pub struct PlaceholderDropped;

#[derive(Event)]
pub struct SpawnFruit {
	pub fruit: Fruit,
	pub position: Vec2,
	pub invincibility: f32,
}

impl SpawnFruit {
	pub fn handle(event: On<Self>, mut commands: Commands) {
		let invincibility = event.invincibility;
		commands.spawn_scene(bsn! {
			@FruitBody {
				@fruit: { event.fruit.clone() },
				@position: { event.position },
			}
			Invincible::new(invincibility)
		});
	}
}

#[derive(Event, Clone, Copy)]
pub struct NewGame;

#[derive(SystemParam)]
pub struct NewGameRes<'w> {
	score: ResMut<'w, Score>,
	fruit_order: ResMut<'w, FruitOrder>,
	next_fruit: ResMut<'w, NextFruit>,
	placeholder_position: Res<'w, PlaceholderPosition>,
}

impl NewGame {
	pub fn handle(
		_: On<Self>,
		mut commands: Commands,
		q_fruits: Query<Entity, With<Fruit>>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
		mut res: NewGameRes,
	) {
		for entity in q_fruits {
			commands.entity(entity).despawn();
		}

		**rng = wyrand::WyRand::new(rand::rng().next_u64()).into();

		res.score.restart();

		let next = res.next_fruit.init(&res.fruit_order, &mut rng);

		commands.queue_spawn_scene(bsn! {
			@PlaceholderFruit {
				@fruit: next,
				@x: { res.placeholder_position.x },
			}
		});
	}
}

#[derive(Event, Clone, Deref)]
pub struct FruitCombo(pub FruitKind);

#[derive(Event, Clone)]
pub struct SetCooldown(pub Option<f32>);

impl SetCooldown {
	pub fn handle(
		event: On<Self>,
		mut commands: Commands,
		mut cooldown: ResMut<FruitDropCooldown>,
		delay: Res<FruitDropDelay>,
		q_placeholders: Query<Entity, With<PlaceholderFruit>>,
	) {
		let cooldown_secs = event.0.unwrap_or_else(|| delay.delay());

		if cooldown_secs > 0.0 {
			info!("event: {:?}, cooldown secs: {cooldown_secs}", event.0);
			cooldown.reset();

			cooldown.set_duration(Duration::from_secs_f32(cooldown_secs));
			for entity in q_placeholders {
				commands.entity(entity).despawn();
			}
		} else if !cooldown.is_finished() {
			cooldown.finish();
		}
	}
}
