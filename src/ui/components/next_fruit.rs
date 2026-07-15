use crate::component_name;
use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct NextFruitUi;

#[derive(SceneComponent, Default, Clone)]
pub struct NextFruitComponent;

impl NextFruitComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(NextFruitComponent) }
			Node {
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
			}
			Children [
				Text("Next") TextColor(Color::BLACK),
				Node { width: px(128), height: px(128) } ImageNode { image: "binary/fruit/cherry.png" } NextFruitUi
			]
		}
	}
}
