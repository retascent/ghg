use std::cell::RefCell;
use std::ops::{Deref};
use std::rc::Rc;
use web_sys::HtmlCanvasElement;
use crate::interaction_core::input_subscriber::{
    FrameInputSubscriber, InputState, KeyState, MouseButton,
    MouseButtonState, MouseMovement, Scroll, SwitchState
};
use crate::render_core::camera::Camera;
use crate::render_core::uniform::Uniform;

use crate::utils::prelude::*;

pub struct Controller {
    input_subscriber: FrameInputSubscriber,
}

impl Controller {
    pub fn new(canvas: HtmlCanvasElement, camera: &Rc<RefCell<Camera>>, terrain_scale: Uniform<f32>) -> Self {
        let mut input_subscriber = FrameInputSubscriber::new(canvas);

        let mouse_move_camera = camera.clone();
        input_subscriber.subscribe_on_mouse_move(Box::new(move |movement: MouseMovement, current_state: InputState| {
            let should_rotate = current_state.is_key_active("ShiftLeft".to_owned())
                || current_state.is_mouse_button_active(MouseButton::Left);

            if should_rotate {
                let mut borrowed_cam = mouse_move_camera.deref().borrow_mut();
                borrowed_cam.orbit_around_target(&nglm::Vec3::zeros(),
                                                 &nglm::vec2(-movement.x as f32, movement.y as f32),
                                                 0.4);
            }
        }));

        input_subscriber.subscribe_on_keyboard_event(Box::new(move |key_states: Vec<KeyState>, _current_state: InputState| {
            let scale_max = 0.1f32;

            key_states.iter().for_each(|k| {
                match k {
                    KeyState { key, state: SwitchState::Pressed } => {
                        match key.as_str() {
                            "Digit1" => { terrain_scale.write_unchecked(0.1 * scale_max) },
                            "Digit2" => { terrain_scale.write_unchecked(0.3 * scale_max) },
                            "Digit3" => { terrain_scale.write_unchecked(0.5 * scale_max) },
                            "Digit4" => { terrain_scale.write_unchecked(0.7 * scale_max) },
                            "Digit5" => { terrain_scale.write_unchecked(0.9 * scale_max) },
                            "Digit6" => { terrain_scale.write_unchecked(1.5 * scale_max) },
                            other => {ghg_log!("{:?}", other)},
                        }
                    }
                    _ => {}
                }
            })
        }));

        let scroll_camera = camera.clone();

        const SCROLL_CLOSEST: f32 = 1.2;
        const SCROLL_FARTHEST: f32 = 4.0;
        const SCROLL_MULTIPLIER: f32 = 0.01;
        const SCROLL_THRESHOLD: f32 = 0.001;
        input_subscriber.subscribe_on_scroll_event(Box::new(move |scroll: Scroll, _current_state: InputState| {
            if scroll.delta_y.abs() > SCROLL_THRESHOLD {
                let mut borrowed_cam = scroll_camera.deref().borrow_mut();
                let last_position = borrowed_cam.position();
                let direction_to_camera = last_position.normalize();
                let last_distance = last_position.magnitude();

                let desired_new_distance = last_distance + scroll.delta_y * SCROLL_MULTIPLIER;
                let capped_new_distance = desired_new_distance.clamp(SCROLL_CLOSEST, SCROLL_FARTHEST);

                let new_position = direction_to_camera * capped_new_distance;

                ghg_log!("last={:?} new={:?}, direction={:?} dist={}", last_position, new_position, direction_to_camera, capped_new_distance);
                borrowed_cam.set_position(new_position);
            }
        }));

        input_subscriber.subscribe_on_mouse_button_event(Box::new(|button_states: Vec<MouseButtonState>, _current_state: InputState| {
            ghg_log!("Mouse button changes: {:?}", button_states);
        }));

        Self {
            input_subscriber,
        }
    }

    pub fn frame(&mut self) {
        self.input_subscriber.frame();
    }
}
