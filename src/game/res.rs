use super::{FruitData, FruitKind};
use crate::game::Fruit;
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
			.init_resource::<MinVelocityFirstDropThresh>()
			.init_resource::<CanSeeNextFruit>();
	}
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Score(usize);

impl Score {
	pub fn restart(&mut self) {
		self.0 = 0;
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct NextFruit(Fruit);

impl Default for NextFruit {
	fn default() -> Self {
		Self(Fruit::from(FruitKind::Cherry))
	}
}

impl NextFruit {
	pub fn init(&mut self, order: &FruitOrder, mut rng: impl Rng) -> Fruit {
		let weighted_fruits = order.chooseable().collect::<Vec<_>>();
		let current_fruit = weighted_fruits
			.choose_weighted(&mut rng, |c| c.1)
			.map(|(f, _)| f)
			.expect("At least one fruit must be chooseable");
		let next_fruit = weighted_fruits
			.choose_weighted(&mut rng, |c| c.1)
			.map(|(f, _)| f)
			.expect("At least one fruit must be chooseable");
		self.0 = Fruit::clone(next_fruit);
		Fruit::clone(current_fruit)
	}

	pub fn get(&mut self, order: &FruitOrder, mut rng: impl Rng) -> Fruit {
		let weighted_fruits = order.chooseable().collect::<Vec<_>>();
		let next_fruit = weighted_fruits
			.choose_weighted(&mut rng, |c| c.1)
			.map(|(f, _)| f)
			.expect("At least one fruit must be chooseable");
		let prev = self.0.clone();
		self.0 = Fruit::clone(next_fruit);
		prev
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct FruitOrder {
	pub order: [Fruit; FruitKind::COUNT],
	pub limit: usize,
}

impl Default for FruitOrder {
	fn default() -> Self {
		Self {
			order: FruitKind::VARIANTS
				.iter()
				.map(|f| Fruit {
					kind: *f,
					data: f.data(),
				})
				.collect::<Vec<_>>()
				.try_into()
				.unwrap(),
			limit: FruitKind::COUNT,
		}
	}
}

impl FruitOrder {
	pub fn set_order(&mut self, order: [FruitKind; FruitKind::COUNT], retain_size: bool) {
		for (i, f) in order.into_iter().enumerate() {
			let original = FruitKind::from_repr(i).unwrap();
			let og_dat = original.data();
			let radius = if retain_size {
				f.radius()
			} else {
				og_dat.radius
			};
			self.order[i] = Fruit {
				kind: f,
				data: FruitData {
					radius,
					weight: og_dat.weight,
					..f.data()
				},
			};
		}
	}

	pub fn chooseable(&self) -> impl Iterator<Item = (&Fruit, usize)> {
		self
			.order
			.iter()
			.filter_map(|f| f.data.weight.map(|w| (f, w)))
			.take(self.limit)
	}

	pub fn kind_array(&self) -> [FruitKind; FruitKind::COUNT] {
		self.order.clone().map(|f| f.kind)
	}

	pub fn next(&self, current: FruitKind) -> Option<&Fruit> {
		self
			.order
			.iter()
			.enumerate()
			.find_map(|(i, f)| (f.kind == current).then_some(i))
			.and_then(|i| self.order.get(i + 1))
	}

	pub fn get(&self, fruit: FruitKind) -> Option<&Fruit> {
		self.order.iter().find(|f| f.kind == fruit)
	}
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct FruitDropDelay {
	pub additional_cooldown: f32,
	upgrades: usize,
	pub max_upgrades: usize,
}

impl Default for FruitDropDelay {
	fn default() -> Self {
		Self {
			additional_cooldown: 0.0,
			upgrades: 0,
			max_upgrades: 0,
		}
	}
}

impl FruitDropDelay {
	pub fn add_upgrades(&mut self, upgrades: usize) {
		self.upgrades += upgrades;
	}

	pub fn delay(&self) -> f32 {
		let additional = if self.max_upgrades != 0 {
			(self.max_upgrades as f32 - self.upgrades as f32) / self.max_upgrades as f32
				* self.additional_cooldown
		} else {
			0.0
		};
		0.5 + additional
	}
}
#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct CanSeeNextFruit(bool);

impl Default for CanSeeNextFruit {
	fn default() -> Self {
		Self(true)
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
