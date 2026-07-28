use crate::{
	component_name,
	game::{ApClient, ApMessage},
	ui::components::AppEditableText,
};
use bevy::{
	color::palettes::tailwind::{SLATE_400, SLATE_800},
	input::mouse::{MouseScrollUnit, MouseWheel},
	input_focus::InputFocus,
	picking::hover::HoverMap,
	prelude::*,
	text::EditableText,
};

const LINE_HEIGHT: f32 = 16.0;

pub struct ApChatPlugin;

impl Plugin for ApChatPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<QueuedMessages>()
			.add_observer(dump_queue)
			.add_systems(Update, (send_scroll_events, handle_submissions))
			.add_systems(FixedUpdate, on_ap_message);
	}
}

#[derive(SceneComponent, Default, Clone)]
pub struct ChatComponent;

impl ChatComponent {
	fn scene() -> impl Scene {
		bsn! {
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				height: px(200),
			}
			Children [
				@ChatBox Node {
					width: percent(100),
					height: percent(85)
				},
				@ChatInput Node {
					width: percent(100),
					height: percent(15)
				},
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct ChatBox;

impl ChatBox {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(ChatBox) }
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				overflow: Overflow::scroll_y(),
			}
			BackgroundColor(SLATE_800)
			on(chatbox_scroll)
		}
	}
}

fn send_scroll_events(
	mut mouse_wheel_reader: MessageReader<MouseWheel>,
	hover_map: Res<HoverMap>,
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut commands: Commands,
) {
	for mouse_wheel in mouse_wheel_reader.read() {
		let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

		if mouse_wheel.unit == MouseScrollUnit::Line {
			delta *= LINE_HEIGHT;
		}

		if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
			std::mem::swap(&mut delta.x, &mut delta.y);
		}

		for pointer_map in hover_map.values() {
			for entity in pointer_map.keys().copied() {
				commands.trigger(Scroll {
					entity,
					kind: ScrollKind::Delta(delta),
				});
			}
		}
	}
}

fn handle_submissions(
	mut commands: Commands,
	input_focus: Res<InputFocus>,
	keyboard: Res<ButtonInput<KeyCode>>,
	mut q_input: Query<&mut EditableText>,
	apclient: Option<ResMut<ApClient>>,
) -> Result {
	if keyboard.just_pressed(KeyCode::Enter)
		&& let Some(focused) = input_focus.get()
		&& let Ok(mut text) = q_input.get_mut(focused)
	{
		let Some(mut apclient) = apclient else {
			commands.queue(ApMessage::new(text.value()));
			text.editor.set_text("");
			return Ok(());
		};

		let msg = text.value().to_string();
		let result = apclient.say(msg);
		if result.is_ok() {
			text.editor.set_text("");
		}
		result?;
	}

	Ok(())
}

#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
	entity: Entity,
	kind: ScrollKind,
}

#[derive(Clone, Copy)]
enum ScrollKind {
	Delta(Vec2),
	ToBottom,
}

fn chatbox_scroll(
	mut scroll: On<Scroll>,
	mut q: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
	let Ok((mut scroll_position, node, computed)) = q.get_mut(scroll.entity) else {
		return;
	};

	let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

	match scroll.kind {
		ScrollKind::Delta(mut delta) => {
			if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
				// Is this node already scrolled all the way in the direction of the scroll?
				let max = if delta.x > 0. {
					scroll_position.x >= max_offset.x
				} else {
					scroll_position.x <= 0.
				};

				if !max {
					scroll_position.x += delta.x;
					// Consume the X portion of the scroll delta.
					delta.x = 0.;
				}
			}

			if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
				// Is this node already scrolled all the way in the direction of the scroll?
				let max = if delta.y > 0. {
					scroll_position.y >= max_offset.y
				} else {
					scroll_position.y <= 0.
				};

				if !max {
					scroll_position.y += delta.y;
					// Consume the Y portion of the scroll delta.
					delta.y = 0.;
				}
			}

			// Stop propagating when the delta is fully consumed.
			if delta == Vec2::ZERO {
				scroll.propagate(false);
			}
		}
		ScrollKind::ToBottom => {
			scroll_position.y = max_offset.y;
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct ChatInput;

impl ChatInput {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(ChatInput) }
			@AppEditableText {
				@name: "Chat Input",
			}
		}
	}
}

#[derive(Resource, Default, Deref, DerefMut)]
struct QueuedMessages(Vec<String>);

fn chat_message(parent: Entity, msg: &str) -> impl Scene {
	bsn! {
		ChildOf(parent)
		Text(msg)
		TextColor(SLATE_400)
		TextFont {
			font_size: FontSize::Px(LINE_HEIGHT)
		}
	}
}

fn on_ap_message(
	mut commands: Commands,
	mut messages: MessageReader<ApMessage>,
	chatbox: Option<Single<Entity, With<ChatBox>>>,
	mut queue: ResMut<QueuedMessages>,
) {
	if let Some(chatbox) = chatbox {
		if !messages.is_empty() {
			commands.delayed().secs(0.5).trigger(Scroll {
				entity: *chatbox,
				kind: ScrollKind::ToBottom,
			});
		}

		for msg in messages.read() {
			commands.queue_spawn_scene(chat_message(*chatbox, &msg.0));
		}
	} else {
		queue.extend(messages.read().map(|m| m.0.clone()));
	}
}

fn dump_queue(event: On<Add, ChatBox>, mut commands: Commands, mut queue: ResMut<QueuedMessages>) {
	for msg in queue.drain(..) {
		commands.queue_spawn_scene(chat_message(event.event_target(), &msg));
	}
	commands.delayed().secs(0.5).trigger(Scroll {
		entity: event.event_target(),
		kind: ScrollKind::ToBottom,
	});
}
