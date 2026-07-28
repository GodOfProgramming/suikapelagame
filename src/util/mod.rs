pub mod events;
pub mod math;

use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn_query<F: QueryFilter>(mut commands: Commands, query: Query<Entity, F>) {
	for entity in query {
		commands.entity(entity).despawn();
	}
}

pub fn entity_exists<F: QueryFilter>(q: Query<(), F>) -> bool {
	!none_of(q)
}

pub fn none_of<F: QueryFilter>(q: Query<(), F>) -> bool {
	q.is_empty()
}
