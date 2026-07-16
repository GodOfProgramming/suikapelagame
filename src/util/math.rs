pub fn position_by_degrees(degrees: f32, offset: f32) -> (f32, f32) {
	let rads = degrees.to_radians();
	let left = offset * (1.0 + rads.cos());
	let bottom = offset * (1.0 + rads.sin());
	(bottom, left)
}
