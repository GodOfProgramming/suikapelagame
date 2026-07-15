use archipelago_rs::Client;
use bevy::prelude::*;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use validator::Validate;

#[derive(Resource, Deref, DerefMut)]
pub struct ApClient(Mutex<Client<SessionData>>);

impl ApClient {
	pub fn new(client: Client<SessionData>) -> Self {
		Self(Mutex::new(client))
	}
}

const MAX_FRUITS: usize = 11;

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
pub struct SessionData {
	deathlink: bool,
	shuffle_fruit: Option<[usize; MAX_FRUITS]>,
	also_shuffle_sizes: bool,
	#[validate(range(min = 0, max = 10))]
	height_upgrade_count: usize,
	#[validate(range(min = 0, max = 10))]
	cooldown_upgrade_count: usize,
	starting_cooldown: usize,
	goal: Goal,
	scoresanity: bool,
	extra_scoresanity: bool,
	difficulty: Difficulty,
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Goal {
	#[default]
	LargestFruit,
	Score,
	Both,
}

#[derive(Debug, Default, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Difficulty {
	VeryEasy,
	Easy,
	EasierThanNormal,
	#[default]
	Normal,
	HarderThanNormal,
	Hard,
	VeryHard,
}
