use crate::{component_name, game::NextFruit};
use bevy::prelude::*;

pub struct NextFruitPlugin;

impl Plugin for NextFruitPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			update_next_fruit.run_if(resource_changed::<NextFruit>),
		);
	}
}

fn update_next_fruit(
	next_fruit: Res<NextFruit>,
	q_next_fruit_images: Query<&mut ImageNode, With<NextFruitImage>>,
	asset_server: Res<AssetServer>,
) {
	for mut img_node in q_next_fruit_images {
		img_node.image = asset_server.load(next_fruit.sprite());
	}
}

#[derive(Component, Default, Clone)]
struct NextFruitImage;

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
				Node { width: px(128), height: px(128) } ImageNode { image: "binary/fruit/cherry.png" } NextFruitImage
			]
		}
	}
}
