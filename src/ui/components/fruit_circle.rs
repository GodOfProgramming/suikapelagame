use bevy::prelude::*;

#[derive(SceneComponent, Default, Clone)]
pub struct FruitCircleComponent;

impl FruitCircleComponent {
	fn scene() -> impl Scene {
		bsn! {
			Node {
				 position_type: PositionType::Absolute,
				 margin: UiRect::all(auto()),
				 max_width: px(320),
				 width: percent(80),
				 aspect_ratio: { Some(1.0) }
			}
			Children [
				@FruitImage {
					@left: percent(75),
					@bottom: percent(93.3),
					@image: "binary/fruit/cherry.png",
				},
				@FruitImage {
					@left: percent(93.3),
					@bottom: percent(75),
					@image: "binary/fruit/strawberry.png",
				},
				@FruitImage {
					@left: percent(100),
					@bottom: percent(50),
					@image: "binary/fruit/grape.png",
				},
				@FruitImage {
					@left: percent(93.3),
					@bottom: percent(25),
					@image: "binary/fruit/dekopon.png",
				},
				@FruitImage {
					@left: percent(75),
					@bottom: percent(6.7),
					@image: "binary/fruit/persimmon.png",
				},
				@FruitImage {
					@left: percent(50),
					@bottom: percent(0),
					@image: "binary/fruit/apple.png"
				},
				@FruitImage {
					@left: percent(25),
					@bottom: percent(6.7)
					@image: "binary/fruit/pear.png",
				},
				@FruitImage {
					@left: percent(6.7),
					@bottom: percent(25),
					@image: "binary/fruit/peach.png",
				},
				@FruitImage {
					@left: percent(0),
					@bottom: percent(50),
					@image: "binary/fruit/pineapple.png",
				},
				@FruitImage {
					@left: percent(6.7),
					@bottom: percent(75),
					@image: "binary/fruit/melon.png",
				},
				@FruitImage {
					@left: percent(25),
					@bottom: percent(93.3),
					@image: "binary/fruit/watermelon.png",
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
}

impl FruitImage {
	fn scene(props: FruitImageProps) -> impl Scene {
		bsn! {
			Node {
				width: percent(20),
				aspect_ratio: { Some(1.0) },
				position_type: PositionType::Absolute,
				left: { props.left },
				bottom: { props.bottom },
				// margin: { UiRect::left(props.left).with_top(-props.bottom) }
			}
			ImageNode {
				image: { props.image },
			}
			UiTransform {
				translation: Val2 {
					x: percent(0),
					y: percent(0),
				}
			}
		}
	}
}
