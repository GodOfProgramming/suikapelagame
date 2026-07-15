use bevy::{color::palettes::tailwind::RED_500, prelude::*};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_observer(UiBorderDebug::on_insert)
			.insert_resource(UiBorderDebugColor(RED_500.into()));
	}
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
struct UiBorderDebug;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct UiBorderDebugColor(Color);

impl UiBorderDebug {
	fn on_insert(
		event: On<Insert, Self>,
		mut commands: Commands,
		mut q_nodes: Query<&mut Node>,
		dbg_color: Res<UiBorderDebugColor>,
	) {
		if let Ok(mut node) = q_nodes.get_mut(event.event_target()) {
			node.border = UiRect::all(px(4));
			commands
				.entity(event.event_target())
				.insert(BorderColor::all(dbg_color.0));
		}
	}
}
