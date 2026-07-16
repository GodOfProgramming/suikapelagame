use bevy::prelude::*;

pub fn send_event<'t>(event: impl Event<Trigger<'t>: Default> + Clone) -> impl FnMut(Commands) {
	move |mut commands: Commands| {
		commands.trigger(event.clone());
	}
}

pub fn send_event_on<'t, E: Event, B: Bundle>(
	event: impl Event<Trigger<'t>: Default> + Clone,
) -> impl FnMut(On<E, B>, Commands) {
	move |_: On<E, B>, mut commands: Commands| {
		commands.trigger(event.clone());
	}
}
