use avian2d::debug_render::{PhysicsDebugPlugin, PhysicsGizmos};
use beditor::{EditorExtension, EditorExtensionPlugin, EditorUi, prelude::uuid};
use bevy::{color::palettes::tailwind::RED_500, ecs::system::SystemParam, prelude::*};
use strum::IntoEnumIterator;

use crate::{
	GameState,
	game::{Fruit, archipelago::ApMessage, termination::lose::LoseHeight},
};

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
struct DebugUiParams<'w, 's> {
	commands: Commands<'w, 's>,
	store: ResMut<'w, GizmoConfigStore>,
	game_state: Res<'w, State<GameState>>,
	next_game_state: ResMut<'w, NextState<GameState>>,
	q_fruits: Query<'w, 's, (Entity, &'static Transform, &'static Fruit)>,
	lose_height: Res<'w, LoseHeight>,

	archipelago_message: Local<'s, String>,
}

impl EditorUi for DebugUi {
	const NAME: &str = "Game Debug UI";

	const ID: uuid::Uuid = uuid::uuid!("c6b86957-acc7-4cb3-95c2-dc226b1104b0");

	const UNIQUE: bool = true;

	type Params<'w, 's> = DebugUiParams<'w, 's>;

	fn spawn(_params: Self::Params<'_, '_>) -> Self {
		Self
	}

	fn ui(&mut self, ui: &mut beditor::prelude::egui::Ui, mut params: Self::Params<'_, '_>) {
		let value = &mut params.store.config_mut::<PhysicsGizmos>().0.enabled;
		ui.checkbox(value, "Show Physics Colliders");

		ui.separator();

		ui.heading("Game State");

		let mut mut_state = *params.game_state.get();
		for state in GameState::iter() {
			if ui
				.selectable_value(&mut mut_state, state, state.to_string())
				.clicked()
			{
				params.next_game_state.set(mut_state);
			}
		}

		ui.separator();

		ui.label(format!("LoseHeight: {}", **params.lose_height));
		ui.indent(ui.id().with("FRUIT_DISPLAY"), |ui| {
			ui.vertical(|ui| {
				for (entity, tf, fruit) in params.q_fruits {
					ui.label(format!(
						"{fruit} ({entity}): {}",
						tf.translation.y + fruit.radius()
					));
				}
			});
		});

		ui.separator();

		ui.label("Trigger Message");
		ui.text_edit_singleline(&mut *params.archipelago_message);
		if ui.button("Send").clicked() {
			params
				.commands
				.queue(ApMessage::new(&*params.archipelago_message));
		}
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
