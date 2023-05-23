use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use web_sys::HtmlCanvasElement;

use crate::application::shaders::ShaderContext;
use crate::interaction_core::input_subscriber::{
	FrameInputSubscriber, InputState, KeyState, MouseButton, MouseButtonState, MouseMovement,
	Scroll, SwitchState, TouchMovement, TouchState,
};
use crate::render_core::animation_params::AnimationParams;
use crate::render_core::camera::Camera;
use crate::render_core::frame_sequencer::FrameGate;
use crate::render_core::uniform;
use crate::render_core::uniform::Uniform;
use crate::utils::prelude::*;

pub struct Controller {
	input_subscriber: FrameInputSubscriber,
}

impl Controller {
	pub fn new(
		canvas: HtmlCanvasElement,
		camera: Rc<RefCell<Camera>>,
		planet_shader: ShaderContext,
		terrain_scale: Uniform<f32>,
		current_month: Rc<Cell<usize>>,
	) -> Self {
		let mut input_subscriber = FrameInputSubscriber::new(canvas);

		let mouse_move_camera = camera.clone();
		input_subscriber.subscribe_on_mouse_move(Box::new(
			move |movement: MouseMovement, current_state: InputState| {
				let should_rotate = current_state.is_key_active("ShiftLeft".to_owned())
					|| current_state.is_mouse_button_active(MouseButton::Left);

				if should_rotate {
					let mut borrowed_cam = mouse_move_camera.deref().borrow_mut();
					borrowed_cam.orbit_around_target(
						&nglm::Vec3::zeros(),
						&nglm::vec2(-movement.x as f32, movement.y as f32),
						0.4,
					);
				}
			},
		));

		let touch_move_camera = camera.clone();
		input_subscriber.subscribe_on_touch_move(Box::new(
			move |all_movement: HashMap<i32, TouchMovement>, current_state: InputState| {
				let all_active = current_state.active_touch_identifiers();

				if all_active.len() == 1 {
					if let Some(movement) = all_movement.get(&all_active[0]) {
						let mut borrowed_cam = touch_move_camera.deref().borrow_mut();
						borrowed_cam.orbit_around_target(
							&nglm::Vec3::zeros(),
							&nglm::vec2(
								-movement.difference.x as f32,
								movement.difference.y as f32,
							),
							0.4,
						);
					}
				} else if all_active.len() == 2 {
					zoom::handle_pinch(
						&touch_move_camera,
						all_movement,
						current_state,
						all_active[0],
						all_active[1],
					);
				}
			},
		));

		input_subscriber.subscribe_on_keyboard_event(Box::new(
			move |key_states: Vec<KeyState>, _current_state: InputState| {
				let scale_max = 0.1f32;

				planet_shader.use_shader();

				key_states.iter().for_each(|k| match k {
					KeyState { key, state: SwitchState::Pressed } => match key.as_str() {
						"Digit1" => terrain_scale.write_unchecked(0.1 * scale_max),
						"Digit2" => terrain_scale.write_unchecked(0.3 * scale_max),
						"Digit3" => terrain_scale.write_unchecked(0.5 * scale_max),
						"Digit4" => terrain_scale.write_unchecked(0.7 * scale_max),
						"Digit5" => terrain_scale.write_unchecked(0.9 * scale_max),
						"Digit6" => terrain_scale.write_unchecked(1.5 * scale_max),
						"ArrowRight" => {
							current_month.replace((current_month.get() + 1) % 12);
						}
						"ArrowLeft" => {
							current_month.replace(((current_month.get() - 1) + 12) % 12);
						}
						other => ghg_log!("{:?}", other),
					},
					_ => {}
				})
			},
		));

		input_subscriber.subscribe_on_scroll_event(zoom::make_scroll_handler(&camera));

		input_subscriber.subscribe_on_mouse_button_event(Box::new(
			|button_states: Vec<MouseButtonState>, _current_state: InputState| {
				ghg_log!("Mouse button changes: {:?}", button_states);
			},
		));

		input_subscriber.subscribe_on_touch_state_event(Box::new(
			|touch_state: HashMap<i32, TouchState>, _current_state: InputState| {
				ghg_log!("Touch state changes: {:?}", touch_state);
			},
		));

		Self { input_subscriber }
	}

	pub fn frame(&mut self) { self.input_subscriber.frame(); }
}

pub async fn controller_frame(
	gate: FrameGate<AnimationParams>,
	canvas: HtmlCanvasElement,
	planet_shader: ShaderContext,
	camera: Rc<RefCell<Camera>>,
	current_month: Rc<Cell<usize>>,
) {
	planet_shader.use_shader();
	let terrain_scale = uniform::init_f32("u_terrainScale", &planet_shader, 0.03);

	let mut controller =
		Controller::new(canvas, camera, planet_shader.clone(), terrain_scale, current_month);

	loop {
		let _params = (&gate).await;
		controller.frame();
	}
}

mod zoom {
	use super::*;
	use crate::interaction_core::input_subscriber::ScrollCallback;

	const SCROLL_MULTIPLIER: f32 = 0.01;
	const SCROLL_THRESHOLD: f32 = 0.001;

	pub fn make_scroll_handler(camera: &Rc<RefCell<Camera>>) -> ScrollCallback {
		let scroll_camera = camera.clone();
		Box::new(move |scroll: Scroll, _current_state: InputState| {
			if scroll.delta_y.abs() > SCROLL_THRESHOLD {
				move_camera(&scroll_camera, scroll.delta_y, SCROLL_MULTIPLIER);
			}
		})
	}

	const ZOOM_CLOSEST: f32 = 1.2;
	const ZOOM_FARTHEST: f32 = 4.0;

	fn move_camera(camera: &Rc<RefCell<Camera>>, distance: f32, multiplier: f32) {
		let mut borrowed_cam = camera.deref().borrow_mut();
		let last_position = borrowed_cam.position();
		let direction_to_camera = last_position.normalize();
		let last_distance = last_position.magnitude();

		let desired_new_distance = last_distance + distance * multiplier;
		let capped_new_distance = desired_new_distance.clamp(ZOOM_CLOSEST, ZOOM_FARTHEST);

		let new_position = direction_to_camera * capped_new_distance;

		borrowed_cam.set_position(new_position);
	}

	const PINCH_MULTIPLIER: f32 = 0.01;

	pub fn handle_pinch(
		camera: &Rc<RefCell<Camera>>,
		all_movement: HashMap<i32, TouchMovement>,
		current_state: InputState,
		a_id: i32,
		b_id: i32,
	) {
		let (maybe_a_pos, maybe_b_pos) = (
			current_state.current_touch_position(a_id),
			current_state.current_touch_position(b_id),
		);
		if maybe_a_pos.is_some() && maybe_b_pos.is_some() {
			let (a_pos, b_pos) = (maybe_a_pos.unwrap(), maybe_b_pos.unwrap());
			let a_prev = if let Some(movement) = all_movement.get(&a_id) {
				a_pos - movement.difference
			} else {
				a_pos
			};

			let b_prev = if let Some(movement) = all_movement.get(&b_id) {
				b_pos - movement.difference
			} else {
				b_pos
			};

			// Casting should be easy...
			let prev_cord = a_prev - b_prev;
			let current_cord = a_pos - b_pos;
			let prev_magnitude = nglm::length(&nglm::vec2(prev_cord.x as f32, prev_cord.y as f32));
			let current_magnitude =
				nglm::length(&nglm::vec2(current_cord.x as f32, current_cord.y as f32));

			let difference = current_magnitude - prev_magnitude;

			move_camera(camera, -1.0 * difference, PINCH_MULTIPLIER);
		}
	}
}
