use avian2d::debug_render::{PhysicsDebugPlugin, PhysicsGizmos};
use beditor::{EditorExtension, EditorExtensionPlugin, EditorUi, prelude::uuid};
use bevy::{color::palettes::tailwind::RED_500, ecs::system::SystemParam, prelude::*};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins((
				PhysicsDebugPlugin,
				EditorExtensionPlugin::<DebugUiExtension>::default(),
			))
			.add_observer(UiBorderDebug::on_insert)
			.insert_resource(UiBorderDebugColor(RED_500.into()));
	}
}

#[derive(Default)]
struct DebugUiExtension;

impl EditorExtension for DebugUiExtension {
	fn build_editor(&self, ctx: &mut beditor::prelude::EditorExtensionContext) {
		ctx.register_ui::<DebugUi>();
	}
}

#[derive(Component, Default)]
struct DebugUi;

#[derive(SystemParam)]
struct DebugUiParams<'w> {
	store: ResMut<'w, GizmoConfigStore>,
}

impl EditorUi for DebugUi {
	const NAME: &str = "Game Debug UI";

	const ID: uuid::Uuid = uuid::uuid!("c6b86957-acc7-4cb3-95c2-dc226b1104b0");

	const UNIQUE: bool = true;

	type Params<'w, 's> = DebugUiParams<'w>;

	fn spawn(_params: Self::Params<'_, '_>) -> Self {
		Self
	}

	fn ui(&mut self, ui: &mut beditor::prelude::egui::Ui, mut params: Self::Params<'_, '_>) {
		let value = &mut params.store.config_mut::<PhysicsGizmos>().0.enabled;
		ui.checkbox(value, "Show Physics Colliders");
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
