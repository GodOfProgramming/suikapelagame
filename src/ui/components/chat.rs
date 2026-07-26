use crate::{component_name, game::ApMessage, ui::components::AppEditableText};
use bevy::{
	color::palettes::tailwind::{SLATE_400, SLATE_800},
	prelude::*,
};

pub struct ApChatPlugin;

impl Plugin for ApChatPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(on_ap_message);
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
				overflow: Overflow::scroll_y()
			}
			BackgroundColor(SLATE_800)
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

fn chat_message(parent: Entity, msg: &str) -> impl Scene {
	bsn! {
		ChildOf(parent)
		Text(msg)
		TextColor(SLATE_400)
	}
}

fn on_ap_message(
	event: On<ApMessage>,
	mut commands: Commands,
	q_chatboxes: Query<Entity, With<ChatBox>>,
) {
	for chatbox in q_chatboxes {
		commands.queue_spawn_scene(chat_message(chatbox, &event.0));
	}
}
