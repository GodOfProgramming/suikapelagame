use bevy::prelude::*;

#[derive(Component, Default, Clone)]
pub struct NextFruitUi;

#[derive(SceneComponent, Default, Clone)]
pub struct NextFruitComponent;

impl NextFruitComponent {
	fn scene() -> impl Scene {
		bsn! {
			Node {
				height: percent(100),
				width: percent(100),
				display: Display::Flex,
				flex_direction: FlexDirection::Column,
				justify_content: JustifyContent::End,
				align_items: AlignItems::Center,
			}
			Children [
				Text("Next"),
				Node { width: px(128), height: px(128) } ImageNode { image: "binary/fruit/cherry.png" } NextFruitUi
			]
		}
	}
}
