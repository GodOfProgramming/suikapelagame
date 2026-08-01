use avian2d::dynamics::rigid_body::LinearVelocity;
use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use rand::seq::{IteratorRandom, SliceRandom};

use crate::{
	component_name,
	game::{
		Fruit, WORLD_DIM,
		cmp::Invincible,
		evt::PlaceholderDropped,
		res::FruitOrder,
		scn::{PlaceholderFruit, WALL_PAD},
		termination::lose::LoseHeight,
	},
	ui::components::NextFruitComponent,
};

pub struct TrapPlugin;

impl Plugin for TrapPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<SecrecyTurns>()
			.init_resource::<BlindnessTurns>()
			.init_resource::<ImpulseForce>()
			.add_observer(ShuffleTrap::handle)
			.add_observer(ThanosTrap::handle)
			.add_observer(SecrecyTrap::handle)
			.add_observer(BlindnessTrap::handle)
			.add_observer(ImpulseTrap::handle)
			.add_observer(on_drop_placeholder);
	}
}

#[derive(Event)]
pub struct ImpulseTrap;

impl ImpulseTrap {
	fn handle(
		_: On<Self>,
		mut commands: Commands,
		q_fruits: Query<(Entity, &Transform, &mut LinearVelocity), With<Fruit>>,
		impulse_force: Res<ImpulseForce>,
	) {
		let bottom = Vec2::new(0.0, -(WORLD_DIM.y / 2.0 - WALL_PAD / 2.0));
		for (entity, tf, mut vel) in q_fruits {
			let dir = tf.translation.xy() - bottom;
			let x = dir * impulse_force.0;
			**vel = 1.0 / 5.0 * x;
			commands.entity(entity).insert(Invincible::new(5.0));
		}
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ImpulseForce(f32);

impl Default for ImpulseForce {
	fn default() -> Self {
		Self(32.0)
	}
}

#[derive(Event)]
pub struct ShuffleTrap;

impl ShuffleTrap {
	fn handle(
		_: On<Self>,
		mut fruit_order: ResMut<FruitOrder>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
	) {
		fruit_order.order.shuffle(&mut rng);
	}
}

#[derive(Event)]
pub struct ThanosTrap;

impl ThanosTrap {
	fn handle(
		_: On<Self>,
		mut commands: Commands,
		q_fruits: Query<Entity, (With<Fruit>, Without<PlaceholderFruit>)>,
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
	) {
		let mut entities = Vec::from_iter(q_fruits.iter());

		for _ in 0..entities.len() / 2 {
			let Some((i, &entity)) = entities.iter().enumerate().choose(&mut rng) else {
				continue;
			};

			entities.swap_remove(i);

			commands.entity(entity).despawn();
		}
	}
}

#[derive(Event)]
pub struct SecrecyTrap(pub usize);

impl SecrecyTrap {
	fn handle(
		event: On<Self>,
		mut turns: ResMut<SecrecyTurns>,
		q_to_hide: Query<&mut Visibility, Or<(With<NextFruitComponent>, With<PlaceholderFruit>)>>,
	) {
		turns.0 = event.0;
		for mut vis in q_to_hide {
			*vis = Visibility::Hidden;
		}
	}
}

#[derive(Resource, Reflect, Default, Deref)]
#[reflect(Resource)]
pub struct SecrecyTurns(usize);

#[derive(Event)]
pub struct BlindnessTrap(pub usize);

impl BlindnessTrap {
	fn handle(
		event: On<Self>,
		mut commands: Commands,
		end_height: Res<LoseHeight>,
		mut blindness: ResMut<BlindnessTurns>,
		blindness_cover: Query<(), With<BlindnessCover>>,
	) {
		blindness.0 = event.0;

		if !blindness_cover.is_empty() {
			return;
		}

		let height = end_height.height();
		let cover = Rect::new(
			-WORLD_DIM.x * 2.0,
			-WORLD_DIM.y * 2.0,
			WORLD_DIM.x * 2.0,
			height,
		);
		let size = cover.size();
		let center = cover.center();
		commands.spawn_scene(bsn! {
			@BlindnessCover {
				@size: size,
				@shift: { center.y },
			}
		});
	}
}

#[derive(Resource, Reflect, Default, Deref)]
#[reflect(Resource)]
pub struct BlindnessTurns(usize);

#[derive(SceneComponent, Default, Clone)]
#[scene(BlindnessCoverParams)]
struct BlindnessCover;

#[derive(Default)]
struct BlindnessCoverParams {
	size: Vec2,
	shift: f32,
}

impl BlindnessCover {
	fn scene(params: BlindnessCoverParams) -> impl Scene {
		bsn! {
			{ component_name!(BlindnessCover) }
			Sprite {
				color: Color::BLACK,
				custom_size: { params.size },
			}
			Transform {
				translation: { Vec3::new(0.0, params.shift, 100.0) }
			}
		}
	}
}

fn on_drop_placeholder(
	_: On<PlaceholderDropped>,
	mut commands: Commands,
	mut secrecy: ResMut<SecrecyTurns>,
	mut blindness: ResMut<BlindnessTurns>,
	q_to_hide: Query<&mut Visibility, Or<(With<NextFruitComponent>, With<PlaceholderFruit>)>>,
	q_covers: Query<Entity, With<BlindnessCover>>,
) {
	// last secrecy turn
	if secrecy.0 == 1 {
		for mut vis in q_to_hide {
			*vis = Visibility::Inherited;
		}
	}

	if blindness.0 == 1 {
		for entity in q_covers {
			commands.entity(entity).despawn();
		}
	}

	secrecy.0 = secrecy.0.saturating_sub(1);
	blindness.0 = blindness.0.saturating_sub(1);
}
