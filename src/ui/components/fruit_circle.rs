use super::FruitImage;
use crate::{
	component_name,
	game::{Fruit, FruitKind, res::FruitOrder},
};
use bevy::prelude::*;
use strum::{EnumCount, VariantArray};

const DEGREE_MAP: [f32; FruitKind::COUNT] = [
	60.0, 30.0, 0.0, 330.0, 300.0, 270.0, 240.0, 210.0, 180.0, 150.0, 120.0,
];

pub struct FruitCirclePlugin;

impl Plugin for FruitCirclePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			on_fruit_order_change.run_if(resource_changed::<FruitOrder>),
		);
	}
}

#[derive(SceneComponent, Default, Clone)]
pub struct FruitCircleComponent;

impl FruitCircleComponent {
	fn scene() -> impl Scene {
		bsn! {
			{ component_name!(FruitCircleComponent) }
			Node {
				 position_type: PositionType::Relative,
				 aspect_ratio: { Some(1.0) }
			}
			Children [
				{ fruit_order_scene(FruitKind::VARIANTS.iter().map(|&f| Fruit::from(f).data.sprite.to_string())) }
			]
		}
	}
}

pub fn fruit_order_scene(fruits: impl Iterator<Item = String>) -> impl SceneList {
	bsn_list![{
		fruits
			.enumerate()
			.map(|(i, sprite)| {
				bsn! {
					@FruitImage {
						@width: 20.0,
						@image: sprite,
						@degrees: { DEGREE_MAP[i] },
					}
				}
			})
			.collect::<Vec<_>>()
	}]
}

fn on_fruit_order_change(
	mut commands: Commands,
	q_fruit_circles: Query<Entity, With<FruitCircleComponent>>,
	fruit_order: Res<FruitOrder>,
) {
	for fruit_circle in q_fruit_circles {
		commands
			.entity(fruit_circle)
			.despawn_children()
			.queue_spawn_related_scenes::<Children>(fruit_order_scene(
				fruit_order.order.iter().map(|f| f.data.sprite.to_string()),
			));
	}
}
