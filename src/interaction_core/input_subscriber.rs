use std::borrow::Borrow;
use std::collections::HashMap;
use std::mem::transmute;
use web_sys::HtmlCanvasElement;
use crate::interaction_core::input_batch::{BatchedInputHandler, InputBatcher};

/// Types passed into callbacks
pub use super::input_batch::{InputState, KeyState, MouseMovement};
pub use super::user_inputs::{MouseButton, MouseButtonState, Scroll, SwitchState};

/// Used to remove a callback after it's unneeded.
pub type Handle = usize;

/// Function to be executed when a mouse movement occurs.
pub type MouseMoveCallback = Box<dyn Fn(MouseMovement, InputState)>;

/// Function to be executed when a keyboard change occurs.
pub type KeyboardEventCallback = Box<dyn Fn(Vec<KeyState>, InputState)>;

/// Function to be executed when a mouse button change occurs.
pub type MouseButtonCallback = Box<dyn Fn(Vec<MouseButtonState>, InputState)>;

/// Function to be executed when a scroll event occurs.
pub type ScrollCallback = Box<dyn Fn(Scroll, InputState)>;

/// Provides a per-frame handler for user input registration.
/// Subscribe to the different types of inputs using callbacks, then use frame() at the start of
/// a frame to handle all of the available inputs, call the registered callbacks, and clear the queue.
/// Order of execution of callbacks of the same type is not guaranteed.
pub struct FrameInputSubscriber {
    input_batcher: InputBatcher,
    keyboard_callbacks: HashMap<Handle, KeyboardEventCallback>,
    mouse_move_callbacks: HashMap<Handle, MouseMoveCallback>,
    mouse_button_callbacks: HashMap<Handle, MouseButtonCallback>,
    scroll_callbacks: HashMap<Handle, ScrollCallback>,
}

impl FrameInputSubscriber {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        Self {
            input_batcher: InputBatcher::new(canvas),
            keyboard_callbacks: Default::default(),
            mouse_move_callbacks: Default::default(),
            mouse_button_callbacks: Default::default(),
            scroll_callbacks: Default::default(),
        }
    }

    pub fn get_current_state(&self) -> InputState {
        self.input_batcher.borrow().get_current_state()
    }

    pub fn subscribe_on_keyboard_event(&mut self, callback: KeyboardEventCallback) -> Handle {
        let handle = self.generate_unique_handle();
        self.keyboard_callbacks.insert(handle, callback);
        handle
    }

    #[allow(dead_code)]
    pub fn unsubscribe_on_keyboard_event(&mut self, handle: Handle) -> Option<KeyboardEventCallback> {
        self.keyboard_callbacks.remove(&handle)
    }

    pub fn subscribe_on_mouse_move<F: 'static + Fn(MouseMovement, InputState)>(&mut self, callback: Box<F>) -> Handle {
        let handle = self.generate_unique_handle();
        self.mouse_move_callbacks.insert(handle, callback);
        handle
    }

    #[allow(dead_code)]
    pub fn unsubscribe_on_mouse_move(&mut self, handle: Handle) -> Option<MouseMoveCallback> {
        self.mouse_move_callbacks.remove(&handle)
    }

    pub fn subscribe_on_mouse_button_event(&mut self, callback: MouseButtonCallback) -> Handle {
        let handle = self.generate_unique_handle();
        self.mouse_button_callbacks.insert(handle, callback);
        handle
    }

    #[allow(dead_code)]
    pub fn unsubscribe_on_mouse_button_event(&mut self, handle: Handle) -> Option<MouseButtonCallback> {
        self.mouse_button_callbacks.remove(&handle)
    }

    pub fn subscribe_on_scroll_event(&mut self, callback: ScrollCallback) -> Handle {
        let handle = self.generate_unique_handle();
        self.scroll_callbacks.insert(handle, callback);
        handle
    }

    #[allow(dead_code)]
    pub fn unsubscribe_on_scroll_event(&mut self, handle: Handle) -> Option<ScrollCallback> {
        self.scroll_callbacks.remove(&handle)
    }

    /// Executes callbacks in this order, if any changes have occurred for each:
    ///  - Keyboard
    ///  - Mouse button
    ///  - Scroll
    ///  - Mouse movement
    /// Thus, all callbacks are guaranteed to have non-empty values when called.
    pub fn frame(&mut self) {
        let current_state = self.get_current_state();
        let input_handler = BatchedInputHandler::new(&mut self.input_batcher);

        let keyboard_changes = input_handler.get_keyboard_changes();
        if !keyboard_changes.is_empty() {
            self.keyboard_callbacks.iter().for_each(|(_handle, cb)| {
                cb(keyboard_changes.clone(), current_state.clone());
            });
        }

        let mouse_button_changes = input_handler.get_mouse_button_changes();
        if !mouse_button_changes.is_empty() {
            self.mouse_button_callbacks.iter().for_each(|(_handle, cb)| {
                cb(mouse_button_changes.clone(), current_state.clone());
            });
        }

        let scroll_changes = input_handler.get_scroll_changes();
        if scroll_changes.is_some() {
            let scroll = scroll_changes.unwrap();
            self.scroll_callbacks.iter().for_each(|(_handle, cb)| {
                cb(scroll.clone(), current_state.clone());
            })
        }

        let mouse_changes = input_handler.get_mouse_movement();
        if mouse_changes.is_some() {
            let movement = mouse_changes.unwrap();
            self.mouse_move_callbacks.iter().for_each(|(_handle, cb)| {
                cb(movement.clone(), current_state.clone());
            });
        }
    }

    fn generate_unique_handle(&self) -> Handle {
        for _i in 0..10 {
            let rand_float = js_sys::Math::random() as f32;
            let handle = unsafe {
                transmute::<f32, Handle>(rand_float)
            };

            if !self.mouse_move_callbacks.contains_key(&handle) {
                return handle;
            }
        }

        panic!("Failed to generate unique handle! Infinite loop.");
    }
}
