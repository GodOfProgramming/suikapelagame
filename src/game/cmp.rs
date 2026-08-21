use bevy::prelude::*;

#[derive(Component, Default, Clone, Deref)]
#[component(storage = "SparseSet")]
pub struct Invincible(f32);

impl Invincible {
	pub fn new(v: impl Into<f32>) -> Self {
		Self(v.into())
	}
}
