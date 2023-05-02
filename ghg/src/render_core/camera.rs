#![allow(dead_code)]

pub struct Camera {
	position: nglm::Vec3,
	forward: nglm::Vec3,
	world_up: nglm::Vec3,
}

pub struct MvpMatrices {
	pub model: nglm::Mat4,
	pub view: nglm::Mat4,
	pub projection: nglm::Mat4,
}

impl Camera {
	pub fn new(position: &nglm::Vec3, target: &nglm::Vec3) -> Self {
		Self {
			position: position.clone(),
			forward: nglm::normalize(&(target - position)),
			world_up: nglm::Vec3::y(),
		}
	}

	pub fn get_perspective_matrices(&self, screen_width: i32, screen_height: i32) -> MvpMatrices {
		let aspect_ratio = (screen_width as f32) / (screen_height as f32);
		let fov_radians = nglm::quarter_pi::<f32>();
		let model: nglm::Mat4 = nglm::identity();
		let view = nglm::look_at(&self.position(), &self.target(), &self.up());
		let mut projection = nglm::perspective(aspect_ratio, fov_radians, 0.1, 10.0);

		projection.data.0[1][1] *= -1.0; // Flip so y points upwards

		MvpMatrices { model, view, projection }
	}

	pub fn position(&self) -> nglm::Vec3 { self.position }

	pub fn set_position(&mut self, new_position: nglm::Vec3) { self.position = new_position; }

	pub fn target(&self) -> nglm::Vec3 { self.position + self.forward }

	pub fn set_target(&mut self, target: nglm::Vec3) {
		self.forward = nglm::normalize(&(target - self.position));
	}

	/// Nothing happens if any component of translation is NaN
	pub fn translate(&mut self, translation: &nglm::Vec3) {
		if !translation[0].is_nan() && !translation[1].is_nan() && !translation[2].is_nan() {
			self.position += translation;
		}
	}

	pub fn move_target_along_sphere(&mut self, sphere_movement: &nglm::Vec2, sensitivity: f32) {
		let radians_movement =
			nglm::vec2(sphere_movement[0].to_radians(), sphere_movement[1].to_radians());
		let sized_movement = radians_movement * sensitivity;
		let yaw = self.yaw_radians() + sized_movement[0];
		let pitch = clamp_pitch_to_up_down(self.pitch_radians() + sized_movement[1]);

		self.forward = nglm::vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
	}

	pub fn orbit_around_target(
		&mut self,
		target: &nglm::Vec3,
		sphere_movement: &nglm::Vec2,
		sensitivity: f32,
	) {
		self.set_target(target.clone());
		let radians_movement =
			nglm::vec2(sphere_movement[0].to_radians(), sphere_movement[1].to_radians());
		let sized_movement = radians_movement * sensitivity;

		// XZ
		let flat_rotation = nglm::rotate(&nglm::identity(), sized_movement.x, &nglm::Vec3::y());
		let mut position4 = nglm::vec3_to_vec4(&self.position());
		position4.w = 1.0;

		// Y
		let axis = self.right();
		let clamped_angle = clamp_vertically(&self.position(), sized_movement.y);
		let vertical_rotation = nglm::rotate(&nglm::identity(), clamped_angle, &axis);
		self.set_position(nglm::vec4_to_vec3(&(flat_rotation * vertical_rotation * position4)));

		// Look again
		self.set_target(target.clone());
	}

	pub fn forward(&self) -> nglm::Vec3 { self.forward }

	pub fn right(&self) -> nglm::Vec3 {
		-1.0 * nglm::normalize(&nglm::cross(&self.world_up, &self.forward))
	}

	pub fn up(&self) -> nglm::Vec3 { -1.0 * nglm::cross(&self.forward, &self.right()) }

	fn forward_xz_plane(&self) -> nglm::Vec2 { nglm::vec2(self.forward[0], self.forward[2]) }

	fn yaw_radians(&self) -> f32 { self.forward[2].atan2(self.forward[0]) }

	fn pitch_radians(&self) -> f32 { self.forward[1].atan2(self.forward_xz_plane().magnitude()) }
}

/// Avoids over-rotation in first-person camera
fn clamp_pitch_to_up_down(pitch: f32) -> f32 {
	const PITCH_STOP: f32 = 1.0e-6;

	if pitch > nglm::half_pi::<f32>() - PITCH_STOP {
		return nglm::half_pi::<f32>() - PITCH_STOP;
	} else if pitch < -nglm::half_pi::<f32>() + PITCH_STOP {
		return -nglm::half_pi::<f32>() + PITCH_STOP;
	}
	pitch
}

/// Avoids over-rotation in third-person orbiting camera
/// NOTE: Assumes the origin is (0, 0, 0)
/// It's not perfect, but it's better than it was...
fn clamp_vertically(position: &nglm::Vec3, vertical_angle_delta: f32) -> f32 {
	const ANGLE_STOP: f32 = 1.0e-3;
	let current_angle = position.normalize().dot(&nglm::Vec3::y()).acos();

	let upper_stop = nglm::pi::<f32>() - ANGLE_STOP;
	let lower_stop = ANGLE_STOP;

	if current_angle + vertical_angle_delta > upper_stop {
		return upper_stop - current_angle;
	} else if current_angle + vertical_angle_delta < lower_stop {
		return lower_stop - current_angle;
	}
	vertical_angle_delta
}
