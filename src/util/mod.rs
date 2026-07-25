pub mod events;
pub mod math;

use bevy::{ecs::query::QueryFilter, prelude::*};

pub fn despawn_query<F: QueryFilter>(mut commands: Commands, query: Query<Entity, F>) {
	for entity in query {
		commands.entity(entity).despawn();
	}
}
