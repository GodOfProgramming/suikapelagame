use super::{Fruit, FruitData};
use bevy::prelude::*;
use enum_map::{EnumMap, enum_map};
use rand::{
	Rng,
	seq::{IndexedRandom, SliceRandom},
};
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
pub struct NextFruit(Fruit);

impl NextFruit {
	pub fn next(&mut self, mut rng: impl Rng) -> Fruit {
		let next_fruit = Fruit::VARIANTS
			.iter()
			.cloned()
			.filter_map(|f| f.weight().map(|w| (f, w)))
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
	pub order: [Fruit; Fruit::COUNT],
	pub shuffle_size: bool,
}

impl Default for FruitOrder {
	fn default() -> Self {
		Self {
			order: Fruit::VARIANTS.try_into().unwrap(),
			shuffle_size: false,
		}
	}
}

impl FruitOrder {
	pub fn order(&self) -> [Fruit; Fruit::COUNT] {
		self.order
	}

	pub fn next(&self, current: Fruit) -> Option<(Fruit, FruitData)> {
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
						.zip(Fruit::from_repr(next))
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
	pub fruit_pops: EnumMap<Fruit, Handle<AudioSource>>,
}

impl FromWorld for StaticAssets {
	fn from_world(world: &mut World) -> Self {
		Self {
			pop: world.load_asset("binary/pop.png"),
			click: world.load_asset("binary/audio/click.mp3"),
			fruit_pops: enum_map! {
				Fruit::Cherry => world.load_asset("binary/audio/cherry.mp3"),
				Fruit::Strawberry => world.load_asset("binary/audio/strawberry.mp3"),
				Fruit::Grapes => world.load_asset("binary/audio/grape.mp3"),
				Fruit::Dekopon => world.load_asset("binary/audio/dekopon.mp3"),
				Fruit::Persimmon => world.load_asset("binary/audio/persimmon.mp3"),
				Fruit::Apple => world.load_asset("binary/audio/apple.mp3"),
				Fruit::Pear => world.load_asset("binary/audio/pear.mp3"),
				Fruit::Peach => world.load_asset("binary/audio/peach.mp3"),
				Fruit::Pineapple => world.load_asset("binary/audio/pineapple.mp3"),
				Fruit::Melon => world.load_asset("binary/audio/melon.mp3"),
				Fruit::Watermelon => world.load_asset("binary/audio/watermelon.mp3"),
			},
		}
	}
}
