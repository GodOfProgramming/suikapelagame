use super::WORLD_DIM;
use crate::{component_name, game::Fruit, ui::RENDER_SCALE};
use avian2d::prelude::*;
use bevy::prelude::*;

pub const WALL_PAD: f32 = 32.0;
const SIDE_POS: f32 = WORLD_DIM.x / 2.0 + WALL_PAD / 2.0;

#[derive(SceneComponent, Default, Clone)]
#[scene(PlaceholderFruitProps)]
pub struct PlaceholderFruit;

#[derive(Default)]
pub struct PlaceholderFruitProps {
	pub fruit: Fruit,
	pub x: f32,
}

impl PlaceholderFruit {
	fn scene(props: PlaceholderFruitProps) -> impl Scene {
		bsn! {
			{ component_name!(PlaceholderFruit) }
			{ template_value(props.fruit) }
			Sprite {
				image: { props.fruit.data.sprite },
				custom_size: { Some(Vec2::splat(props.fruit.data.radius)) }
			}
			Transform {
				translation: { Vec3::new(props.x, WORLD_DIM.y * 1.8, 1.0) },
				scale: Vec3::from((Vec2::splat(RENDER_SCALE), 1.0)),
			}
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(FruitBodyProps)]
pub struct FruitBody;

#[derive(Default)]
pub struct FruitBodyProps {
	pub fruit: Fruit,
	pub position: Vec2,
}

impl FruitBody {
	fn scene(props: FruitBodyProps) -> impl Scene {
		bsn! {
			Name::new(props.fruit.kind.to_string())
			Collider::circle(props.fruit.data.radius / 2.0)
			CollisionEventsEnabled
			SweptCcd
			Mass({props.fruit.data.radius})
			Restitution {
				coefficient: 0.1,
			}
			Friction {
				dynamic_coefficient: 0.06,
				static_coefficient: 0.06,
			}
			Sprite {
				image: { props.fruit.data.sprite },
				custom_size: { Some(Vec2::splat(props.fruit.data.radius)) }
			}
			Transform {
				translation: { Vec3::from((props.position, 1.0)) },
				scale: Vec3::from((Vec2::splat(RENDER_SCALE), 1.0)),
			}
			template_value(props.fruit)
			template_value(RigidBody::Dynamic)
		}
	}
}

#[derive(SceneComponent, Default, Clone)]
#[scene(AreaBoundProps)]
struct AreaBound;

#[derive(Default)]
struct AreaBoundProps {
	name: String,
	location: Vec2,
	size: Vec2,
}

impl AreaBound {
	fn scene(props: AreaBoundProps) -> impl Scene {
		const WALL_COLOR: Color = Color::srgb_u8(0xff, 0xee, 0xdb);
		bsn! {
			Name::new(props.name)
			template_value(RigidBody::Static)
			Collider::rectangle(1.0, 1.0)
			Sprite {
				color: WALL_COLOR,
			}
			Transform {
				translation: { Vec3::from((props.location * RENDER_SCALE, 1.0)) },
				scale: { Vec3::from((props.size * RENDER_SCALE, 1.0)) }
			}
		}
	}
}

pub fn level() -> impl Scene {
	bsn! {
		Name::new("Level")
		Transform Visibility
		Children [
			@AreaBound {
				@name: "Left",
				@location: Vec2 {
					x: { -SIDE_POS },
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { WORLD_DIM.y },
				}
			},
			@AreaBound {
				@name: "Right",
				@location: Vec2 {
					x: SIDE_POS,
					y: 0.0,
				},
				@size: Vec2 {
					x: WALL_PAD,
					y: { WORLD_DIM.y },
				}
			},
			@AreaBound {
				@name: "Top",
				@location: Vec2 {
					x: 0.0,
					y: {  WORLD_DIM.y / 2.0 - WALL_PAD / 2.0  },
				},
				@size: Vec2 {
					x:  { WORLD_DIM.x + WALL_PAD * 2.0 },
					y:  WALL_PAD
				}
			},
			@AreaBound {
				@name: "Bottom",
				@location: Vec2 {
					x: 0.0,
					y: { -( WORLD_DIM.y / 2.0 - WALL_PAD / 2.0 ) },
				},
				@size: Vec2 {
					x:  { WORLD_DIM.x + WALL_PAD * 2.0 },
					y:  WALL_PAD
				}
			}
		]
	}
}
