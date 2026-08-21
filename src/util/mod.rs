pub mod events;
pub mod math;

use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn_query<F: QueryFilter>(mut commands: Commands, query: Query<Entity, F>) {
	for entity in query {
		commands.entity(entity).despawn();
	}
}

pub fn component_exists<C: Component>(q: Query<(), With<C>>) -> bool {
	!q.is_empty()
}

pub fn component_added<C: Component>(q: Query<(), Added<C>>) -> bool {
	!q.is_empty()
}
