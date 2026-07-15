use bevy::prelude::*;

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
					@left: percent(75),
					@bottom: percent(93.3),
					@image: "binary/fruit/cherry.png",
					@degrees: 60.0,
				},
				@FruitImage {
					@left: percent(93.3),
					@bottom: percent(75),
					@image: "binary/fruit/strawberry.png",
					@degrees: 30.0,
				},
				@FruitImage {
					@left: percent(100),
					@bottom: percent(50),
					@image: "binary/fruit/grape.png",
					@degrees: 0.0,
				},
				@FruitImage {
					@left: percent(93.3),
					@bottom: percent(25),
					@image: "binary/fruit/dekopon.png",
					@degrees: 330.0,
				},
				@FruitImage {
					@left: percent(75),
					@bottom: percent(6.7),
					@image: "binary/fruit/persimmon.png",
					@degrees: 300.0,
				},
				@FruitImage {
					@left: percent(50),
					@bottom: percent(0),
					@image: "binary/fruit/apple.png"
					@degrees: 270.0,
				},
				@FruitImage {
					@left: percent(25),
					@bottom: percent(6.7)
					@image: "binary/fruit/pear.png",
					@degrees: 240.0,
				},
				@FruitImage {
					@left: percent(6.7),
					@bottom: percent(25),
					@image: "binary/fruit/peach.png",
					@degrees: 210.0,
				},
				@FruitImage {
					@left: percent(0),
					@bottom: percent(50),
					@image: "binary/fruit/pineapple.png",
					@degrees: 180.0,
				},
				@FruitImage {
					@left: percent(6.7),
					@bottom: percent(75),
					@image: "binary/fruit/melon.png",
					@degrees: 150.0,
				},
				@FruitImage {
					@left: percent(25),
					@bottom: percent(93.3),
					@image: "binary/fruit/watermelon.png",
					@degrees: 120.0,
				}
			]
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(FruitImageProps)]
struct FruitImage;

#[derive(Default)]
struct FruitImageProps {
	left: Val,
	bottom: Val,
	image: String,
	degrees: f32,
}

impl FruitImage {
	fn scene(props: FruitImageProps) -> impl Scene {
		const WIDTH: f32 = 20.0;
		const OFFSET: f32 = 50.0 - WIDTH / 2.0;

		let rads = props.degrees.to_radians();
		let left = OFFSET * (1.0 + rads.cos());
		let bottom = OFFSET * (1.0 + rads.sin());

		bsn! {
			Node {
				width: percent(WIDTH),
				aspect_ratio: { Some(1.0) },
				position_type: PositionType::Absolute,
				left: { percent(left) },
				bottom: { percent(bottom) },
				// margin: { UiRect::left(props.left).with_top(-props.bottom) }
			}
			ImageNode {
				image: { props.image },
			}
		}
	}
}
