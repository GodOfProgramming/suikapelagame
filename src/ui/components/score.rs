use bevy::prelude::*;

#[derive(SceneComponent, Default, Clone)]
pub struct ScoreComponent;

impl ScoreComponent {
	fn scene() -> impl Scene {
		bsn! {}
	}
}
