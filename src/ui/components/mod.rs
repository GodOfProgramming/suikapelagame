pub mod chat;
pub mod fruit_circle;
pub mod login_form;
pub mod lose_popup;
pub mod next_fruit;
pub mod saveload;
pub mod score;
pub mod start_menu;
pub mod win_popup;

use crate::{GameState, util::math};
use bevy::{
	color::palettes::{
		css::DARK_GREY,
		tailwind::{SLATE_200, SLATE_300, SLATE_500},
	},
	ecs::query::QueryFilter,
	prelude::*,
	text::{EditableText, TextCursorStyle},
	window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};

pub use {
	fruit_circle::FruitCircleComponent, login_form::LoginFormComponent,
	next_fruit::NextFruitComponent, score::ScoreComponent,
};

#[derive(SceneComponent, Default, Clone)]
#[scene(AppEditableTextProps)]
pub struct AppEditableText;

#[derive(Default)]
pub struct AppEditableTextProps {
	name: String,
	placeholder_text: String,
}

impl AppEditableText {
	fn scene(props: AppEditableTextProps) -> impl Scene {
		let name = Name::new(props.name);

		let mut text = EditableText::new(props.placeholder_text);
		text.visible_width = Some(10.);
		text.allow_newlines = false;

		bsn! {
			Node {
				border: { px(2).all() }
			}
			BorderColor::from(Color::from(SLATE_300))
			template_value(name)
			template_value(text)
			TextLayout::no_wrap()
			TextCursorStyle::default()
			TextColor(SLATE_200)
			BackgroundColor(DARK_GREY)
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(FruitImageProps)]
pub struct FruitImage;

#[derive(Default)]
pub struct FruitImageProps {
	/// Only as a percent
	pub width: f32,
	pub image: String,
	pub degrees: f32,
}

impl FruitImage {
	fn scene(props: FruitImageProps) -> impl Scene {
		let offset = 50.0 - props.width / 2.0;

		let (bottom, left) = math::position_by_degrees(props.degrees, offset);

		bsn! {
			Node {
				width: percent(props.width),
				aspect_ratio: { Some(1.0) },
				position_type: PositionType::Absolute,
				left: { percent(left) },
				bottom: { percent(bottom) },
			}
			ImageNode {
				image: { props.image },
			}
		}
	}
}

fn column(children: impl SceneList) -> impl Scene {
	bsn! {
		Node {
			display: Display::Flex,
			flex_direction: FlexDirection::Column,
		}
		Children [
			{children}
		]
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(StatePopupProps)]
struct StatePopup;

struct StatePopupProps {
	parent: Entity,
	header: String,
	header_color: Color,
	border_color: Color,
}

impl Default for StatePopupProps {
	fn default() -> Self {
		Self {
			parent: Entity::PLACEHOLDER,
			header: default(),
			header_color: default(),
			border_color: default(),
		}
	}
}

impl StatePopup {
	fn scene(props: StatePopupProps) -> impl Scene {
		bsn! {
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				position_type: PositionType::Absolute,
				border_radius: BorderRadius::all(px(8)),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				border: UiRect::all(px(4)),
				aspect_ratio: { Some(3.0) },
				width: percent(50),
			}
			BorderColor::all(props.border_color)
			BackgroundColor(Color::srgb_u8(0xff, 0xed, 0xda))
			ZIndex(1)
			ChildOf({ props.parent })
			Children [
				Text::new(props.header.clone())
				TextColor({ props.header_color })
				TextFont {
					font_size: FontSize::Px(36.0),
				},
				@ClickableButton
				Node {
					display: Display::Flex,
					align_items: AlignItems::Center,
					justify_content: JustifyContent::Center,
					border_radius: BorderRadius::all(px(8)),
					width: px(296),
					height: px(64),
				}
				BackgroundColor(Color::srgb_u8(0xff, 0x87, 0x00))
				BoxShadow::new(
					Color::srgb_u8(0x75, 0x75, 0x75),
					px(1.5),
					px(3),
					px(3.5),
					px(0),
				)
				Children [
					Text::new("Start Game")
					TextFont {
						weight: FontWeight(900),
						font_size: FontSize::Px(32.0),
					},
				]
				on(
					|_event: On<Pointer<Click>>, mut game_state: ResMut<NextState<GameState>>| {
						game_state.set(GameState::Gameplay);
					},
				)
			]
		}
	}
}

fn spawn_state_popup<F: QueryFilter>(
	header: impl Into<String>,
	header_color: Color,
	border_color: Color,
) -> impl FnMut(Commands, Single<Entity, F>) {
	let header = header.into();
	move |mut commands, game_view| {
		commands.queue_spawn_scene(bsn! {
			@StatePopup {
				@parent: { *game_view },
				@header: { header.clone() },
				@header_color: header_color,
				@border_color: border_color,
			}
		});
	}
}

#[derive(SceneComponent, Default, Clone)]
pub struct AppButton;

impl AppButton {
	fn scene() -> impl Scene {
		bsn! {
			@ClickableButton BackgroundColor(SLATE_500)
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
struct ClickableButton;

impl ClickableButton {
	fn scene() -> impl Scene {
		bsn! {
			Button
			Node {
				border_radius: BorderRadius::all(px(8)),
				margin: { UiRect::top(px(8)).with_bottom(px(8)) },
			}
			on(
				|_event: On<Pointer<Click>>,
					mut commands: Commands,
					window: Single<Entity, With<PrimaryWindow>>| {
					commands
						.entity(*window)
						.insert(CursorIcon::System(SystemCursorIcon::default()));
				},
			)
			on(
				|_event: On<Pointer<Enter>>,
					mut commands: Commands,
					window: Single<Entity, With<PrimaryWindow>>| {
					commands
						.entity(*window)
						.insert(CursorIcon::System(SystemCursorIcon::Pointer));
				},
			)
			on(
				|_event: On<Pointer<Leave>>,
					mut commands: Commands,
					window: Single<Entity, With<PrimaryWindow>>| {
					commands
						.entity(*window)
						.insert(CursorIcon::System(SystemCursorIcon::default()));
				},
			)
		}
	}
}

pub fn appzone(children: impl SceneList) -> impl Scene {
	bsn! {
		Node {
			padding: UiRect::all(px(16)),
			border_radius: BorderRadius::all(px(16))
		}
		BackgroundColor({ Color::srgb_u8(0xff, 0xea, 0xce) })
		Children [
			{ children }
		]
	}
}
