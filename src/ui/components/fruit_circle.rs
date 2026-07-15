use bevy::prelude::*;

use super::FruitImage;
use crate::component_name;

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
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/cherry.png",
					@degrees: 60.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/strawberry.png",
					@degrees: 30.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/grape.png",
					@degrees: 0.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/dekopon.png",
					@degrees: 330.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/persimmon.png",
					@degrees: 300.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/apple.png"
					@degrees: 270.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/pear.png",
					@degrees: 240.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/peach.png",
					@degrees: 210.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/pineapple.png",
					@degrees: 180.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/melon.png",
					@degrees: 150.0,
				},
				@FruitImage {
					@width: 20.0,
					@image: "binary/fruit/watermelon.png",
					@degrees: 120.0,
				}
			]
		}
	}
}
