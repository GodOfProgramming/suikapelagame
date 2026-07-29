use bevy::prelude::*;

#[derive(Component, Default, Clone, Deref)]
pub struct FruitValue(usize);

impl FruitValue {
	pub fn new(v: impl Into<usize>) -> Self {
		Self(v.into())
	}
}

#[derive(Component, Default, Clone, Deref)]
#[component(storage = "SparseSet")]
pub struct Invincible(f32);

impl Invincible {
	pub fn new(v: impl Into<f32>) -> Self {
		Self(v.into())
	}
}
