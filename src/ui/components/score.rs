use bevy::prelude::*;

use crate::{component_name, game::Score};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, update_score.run_if(resource_changed::<Score>));
	}
}

fn update_score(score: Res<Score>, q_text: Query<&mut Text, With<ScoreText>>) {
	for mut txt in q_text {
		txt.0 = score.to_string();
	}
}

#[derive(Component, Default, Clone)]
struct ScoreText;

#[derive(SceneComponent, Default, Clone)]
pub struct ScoreComponent;

impl ScoreComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(ScoreComponent) }
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
			}
			Children [
				title(),
				score(),
			]
		}
	}
}

fn title() -> impl Scene {
	bsn! {
		Text("Score")
		TextColor({Color::srgb_u8(0x66, 0x66, 0x66)})
		template_value(FontSize::Px(16.0))
	}
}

fn score() -> impl Scene {
	bsn! {
		Text("0")
		ScoreText
		TextFont {
			weight: FontWeight(900)
			font_size: FontSize::Px(84.0),
		}
		TextShadow {
			color: {Color::srgb_u8(0xff, 0x53, 0x00)},
			offset: Vec2 {
				x: 3.0,
				y: 3.0,
			}
		}
	}
}
