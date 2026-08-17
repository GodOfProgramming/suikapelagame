use crate::game::{
	Fruit,
	cmp::Invincible,
	res::{FruitDropDelay, FruitOrder, NextFruit, PlaceholderPosition, Score, StaticAssets},
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

#[derive(SystemParam)]
pub struct DropPlaceholderRes<'w> {
	next_fruit: ResMut<'w, NextFruit>,
	static_assets: Res<'w, StaticAssets>,
	placeholder_pos: Res<'w, PlaceholderPosition>,
	drop_delay: Res<'w, FruitDropDelay>,
	order: Res<'w, FruitOrder>,
	time: Res<'w, Time>,
}

impl DropPlaceholder {
	pub fn handle(
		_: On<Self>,
		mut commands: Commands,
		placeholder: Single<(Entity, &Fruit, &Transform), With<PlaceholderFruit>>,
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
					@fruit: { fruit.clone() },
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
			.secs(res.drop_delay.delay())
			.queue_spawn_scene(bsn! {
				@PlaceholderFruit {
					@fruit: fruit,
					@x: { res.placeholder_pos.x },
				}
			});

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
