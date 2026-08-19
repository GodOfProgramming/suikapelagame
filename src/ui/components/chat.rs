use crate::{
	component_name,
	game::archipelago::{ApClient, msg::ApMessage},
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
use clap::Parser;

const LINE_HEIGHT: f32 = 16.0;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<QueuedMessages>()
			.add_observer(dump_queue)
			.add_observer(cmd::Args::handle)
			.add_systems(Update, (send_scroll_events, on_submit))
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
				info!("send scroll to {entity}");
				commands.trigger(Scroll {
					entity,
					kind: ScrollKind::Delta(delta),
				});
			}
		}
	}
}

fn on_submit(
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
		let msg = text.value().to_string();

		if let Some(msg) = msg.strip_prefix("/")
			&& let Some(words) = shlex::split(msg)
		{
			match cmd::Args::try_parse_from(words) {
				Ok(args) => {
					commands.trigger(args);
				}
				Err(err) => {
					commands.write_message(ApMessage::new(err.render()));
				}
			}
			text.editor.set_text("");
			return Ok(());
		}

		let Some(mut apclient) = apclient else {
			commands.queue(ApMessage::new(msg));
			text.editor.set_text("");
			return Ok(());
		};

		let result = apclient.say(msg);
		if result.is_ok() {
			text.editor.set_text("");
		}
		result?;
	}

	Ok(())
}

// Scroll logic from bevy examples

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
	info!("received scroll");
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
struct QueuedMessages(Vec<ApMessage>);

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

fn rich_chat_message(parent: Entity, msg: &str) -> impl Bundle {
	(ChildOf(parent), bevy_simple_rich_text::RichText::new(msg))
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
			match msg {
				ApMessage::Text(msg) => {
					commands.queue_spawn_scene(chat_message(*chatbox, msg));
				}
				ApMessage::Rich(msg) => {
					commands.spawn(rich_chat_message(*chatbox, msg));
				}
			}
		}
	} else {
		queue.extend(messages.read().cloned());
	}
}

fn dump_queue(event: On<Add, ChatBox>, mut commands: Commands, mut queue: ResMut<QueuedMessages>) {
	for msg in queue.drain(..) {
		match msg {
			ApMessage::Text(msg) => {
				commands.queue_spawn_scene(chat_message(event.event_target(), &msg));
			}
			ApMessage::Rich(msg) => {
				commands.spawn(rich_chat_message(event.event_target(), &msg));
			}
		}
	}
	commands.delayed().secs(0.5).trigger(Scroll {
		entity: event.event_target(),
		kind: ScrollKind::ToBottom,
	});
}

mod cmd {
	use bevy::prelude::*;
	use bevy_defer::AsyncCommandsExtension;
	use clap::{Parser, Subcommand};
	use itertools::Itertools;

	use crate::game::{
		archipelago::{ApClient, msg::ApMessage},
		evt::NewGame,
	};

	#[derive(Event, Parser)]
	#[command(multicall = true)]
	pub struct Args {
		#[command(subcommand)]
		command: Command,
	}

	#[derive(Subcommand)]
	pub enum Command {
		/// Request an archipelago sync
		Sync,
		/// Connect to a new server
		Connect {
			name: String,
			url: String,
			#[arg(short, long)]
			password: Option<String>,
		},
		/// Start a new game
		NewGame,
		/// Location Information
		Locations {
			#[arg(short, long, help = "Show checked locations")]
			checked: bool,
			#[arg(short, long, help = "Show unchecked locations")]
			unchecked: bool,
		},
		Items,
		Cheat {
			#[arg(short, long, help = "Force release this item location")]
			location: Option<i64>,
		},
	}

	impl Args {
		pub fn handle(event: On<Self>, mut commands: Commands, apclient: Option<ResMut<ApClient>>) {
			match &event.command {
				Command::Sync => {
					let Some(mut apclient) = apclient else {
						return;
					};

					info!("Requesting sync...");

					let Err(err) = apclient.sync() else {
						commands.write_message(ApMessage::new("Requesting sync..."));
						return;
					};

					commands.write_message(ApMessage::error(format!("Failed to sync: {err}")));
				}
				Command::Connect {
					url,
					password,
					name,
				} => {
					let url = url.clone();
					let name = name.clone();
					let password = password.clone();
					commands.spawn_task(|| async {
						ApClient::login(url, name, password).await;
						Ok(())
					});
				}
				Command::NewGame => {
					commands.trigger(NewGame);
				}
				Command::Locations { checked, unchecked } => {
					let Some(apclient) = apclient else {
						return;
					};

					match (checked, unchecked) {
						(true, false) => {
							let iter = apclient
								.checked_locations()
								.sorted_by_key(|l| l.id())
								.map(|l| format!("{} ({})", l.name(), l.id()));
							let msg = itertools::join(iter, "\n");
							commands.write_message(ApMessage::new(msg));
						}
						(false, true) => {
							let iter = apclient
								.unchecked_locations()
								.sorted_by_key(|l| l.id())
								.map(|l| format!("{} ({})", l.name(), l.id()));
							let msg = itertools::join(iter, "\n");
							commands.write_message(ApMessage::new(msg));
						}
						_ => {
							let checked = apclient
								.checked_locations()
								.sorted_by_key(|l| l.id())
								.map(|l| format!(" {} ({})", l.name(), l.id()));
							let unchecked = apclient
								.unchecked_locations()
								.sorted_by_key(|l| l.id())
								.map(|l| format!(" {} ({})", l.name(), l.id()));
							let iter = std::iter::once(String::from("Checked:"))
								.chain(checked)
								.chain(std::iter::once(String::from("Unchecked:")))
								.chain(unchecked);
							let msg = itertools::join(iter, "\n");
							commands.write_message(ApMessage::new(msg));
						}
					}
				}
				Command::Items => {
					let Some(apclient) = apclient else {
						return;
					};

					let iter = apclient
						.received_items()
						.iter()
						.map(|i| format!("{} ({})", i.item().name(), i.item().id()));
					let msg = itertools::join(iter, "\n");
					commands.write_message(ApMessage::new(msg));
				}
				Command::Cheat { location } => {
					let Some(mut apclient) = apclient else {
						return;
					};

					if let Some(location) = location
						&& let Err(err) = apclient.mark_checked([*location])
					{
						commands.write_message(ApMessage::error(err));
					}
				}
			}
		}
	}
}
