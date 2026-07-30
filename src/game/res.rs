use super::{FruitData, FruitKind};
use bevy::prelude::*;
use enum_map::{EnumMap, enum_map};
use rand::{Rng, seq::IndexedRandom};
use strum::{EnumCount, VariantArray};

pub struct ResPlugin;

impl Plugin for ResPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<PlaceholderPosition>()
			.init_resource::<NextFruit>()
			.init_resource::<StaticAssets>()
			.init_resource::<Score>()
			.init_resource::<FruitDropDelay>()
			.init_resource::<FruitOrder>()
			.init_resource::<MinVelocityFirstDropThresh>();
	}
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Score(usize);

impl Score {
	pub fn restart(&mut self) {
		self.0 = 0;
	}
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct NextFruit(FruitKind);

impl NextFruit {
	pub fn get(&mut self, order: &FruitOrder, mut rng: impl Rng) -> FruitKind {
		let next_fruit = order
			.chooseable()
			.collect::<Vec<_>>()
			.choose_weighted(&mut rng, |c| c.1)
			.map(|(f, _)| *f)
			.unwrap_or_default();
		let prev = self.0;
		self.0 = next_fruit;
		prev
	}
}

#[derive(Resource)]
pub struct FruitOrder {
	pub order: [FruitKind; FruitKind::COUNT],
	pub shuffle_size: bool,
}

impl Default for FruitOrder {
	fn default() -> Self {
		Self {
			order: FruitKind::VARIANTS.try_into().unwrap(),
			shuffle_size: false,
		}
	}
}

impl FruitOrder {
	pub fn chooseable(&self) -> impl Iterator<Item = (FruitKind, usize)> {
		self
			.order
			.into_iter()
			.filter_map(|f| f.weight().map(|w| (f, w)))
	}

	pub fn next(&self, current: FruitKind) -> Option<(FruitKind, FruitData)> {
		self
			.order
			.iter()
			.enumerate()
			.find_map(|(i, f)| (*f == current).then_some(i))
			.and_then(|i| {
				if self.shuffle_size {
					self.order.get(i + 1).map(|f| (*f, f.data()))
				} else {
					let next = i + 1;
					self
						.order
						.get(next)
						.zip(FruitKind::from_repr(next))
						.map(|(order_next, real_next)| {
							let data = real_next.data();
							(
								*order_next,
								FruitData {
									sprite: order_next.sprite(),
									..data
								},
							)
						})
				}
			})
	}
}

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct FruitDropDelay(f32);

impl Default for FruitDropDelay {
	fn default() -> Self {
		Self(0.5)
	}
}

#[derive(Resource, Reflect, Deref)]
#[reflect(Resource)]
pub struct MinVelocityFirstDropThresh(f32);

impl Default for MinVelocityFirstDropThresh {
	fn default() -> Self {
		Self(10.0)
	}
}

#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct PlaceholderPosition(pub Vec2);

#[derive(Resource)]
pub struct StaticAssets {
	pub pop: Handle<Image>,
	pub click: Handle<AudioSource>,
	pub fruit_pops: EnumMap<FruitKind, Handle<AudioSource>>,
}

impl FromWorld for StaticAssets {
	fn from_world(world: &mut World) -> Self {
		Self {
			pop: world.load_asset("binary/pop.png"),
			click: world.load_asset("binary/audio/click.mp3"),
			fruit_pops: enum_map! {
				FruitKind::Cherry => world.load_asset("binary/audio/cherry.mp3"),
				FruitKind::Strawberry => world.load_asset("binary/audio/strawberry.mp3"),
				FruitKind::Grapes => world.load_asset("binary/audio/grape.mp3"),
				FruitKind::Dekopon => world.load_asset("binary/audio/dekopon.mp3"),
				FruitKind::Persimmon => world.load_asset("binary/audio/persimmon.mp3"),
				FruitKind::Apple => world.load_asset("binary/audio/apple.mp3"),
				FruitKind::Pear => world.load_asset("binary/audio/pear.mp3"),
				FruitKind::Peach => world.load_asset("binary/audio/peach.mp3"),
				FruitKind::Pineapple => world.load_asset("binary/audio/pineapple.mp3"),
				FruitKind::Melon => world.load_asset("binary/audio/melon.mp3"),
				FruitKind::Watermelon => world.load_asset("binary/audio/watermelon.mp3"),
			},
		}
	}
}
