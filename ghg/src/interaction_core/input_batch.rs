use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, TouchEvent, WheelEvent};

use crate::interaction_core::user_inputs::{
	KeyCodeState, LogicalCursorPosition, LogicalTouchPosition, MouseButton, MouseButtonState,
	Scroll, SwitchState, TouchState, UserInput,
};
use crate::utils::prelude::*;

pub type KeyState = KeyCodeState<KeyCode>;

// The handlers in this struct are just for lifetime, not really used after
// register
/// Subscribes to input events from the front end, and provides batches of
/// events which have occurred since the last time it was cleared. Generally,
/// this is used for batching inputs between frames, then handling them all at
/// once. To clear the batch, create a FrameInputHandler, which will clear
/// the batch on destruction.
#[allow(dead_code)]
pub struct InputBatcher {
	current_state: Rc<RefCell<InputState>>,
	previous_state: InputState,

	mouse_move_handler: MouseEventHandler,
	mouse_button_handlers: MouseButtonHandlers,
	mouse_scroll_handler: MouseScrollHandler,
	keyboard_handlers: KeyboardEventHandlers,
	touch_position_handler: TouchPositionHandler,
	touch_state_handlers: TouchStateHandlers,
}

impl InputBatcher {
	pub fn new(canvas: HtmlCanvasElement) -> Self {
		let current_state = Rc::new(RefCell::new(InputState::new()));
		let mouse_move_handler = add_mouse_move_handler(&canvas, current_state.clone());
		let mouse_button_handlers = add_mouse_button_handlers(&canvas, &current_state);
		let mouse_scroll_handler = add_mouse_scroll_handler(&canvas, current_state.clone());
		let keyboard_handlers = add_keyboard_handlers(&canvas, &current_state);
		let touch_position_handler = add_touch_position_handler(&canvas, current_state.clone());
		let touch_state_handlers = add_touch_state_handlers(&canvas, &current_state);

		Self {
			current_state,
			previous_state: InputState::new(),
			mouse_move_handler,
			mouse_button_handlers,
			mouse_scroll_handler,
			keyboard_handlers,
			touch_position_handler,
			touch_state_handlers,
		}
	}

	pub fn get_current_state(&self) -> InputState { self.current_state.borrow().deref().clone() }

	// Not for direct use. Use the FrameInputInterpreter to do this at the end
	// of the frame.
	fn store_last(&mut self) {
		self.previous_state = self.current_state.borrow().clone();
		self.current_state.deref().borrow_mut().unhandled_changes.clear();
	}
}

pub struct BatchedInputHandler<'a> {
	input_batcher: &'a mut InputBatcher,
}

impl<'a> BatchedInputHandler<'a> {
	pub fn new(input_batcher: &'a mut InputBatcher) -> Self { Self { input_batcher } }

	// TODO: These should use a clearer diff algorithm with the previous frame.
	pub fn get_mouse_movement(&self) -> Option<MouseMovement> {
		self.input_batcher.current_state.borrow().unhandled_changes.mouse_movement
	}

	pub fn get_keyboard_changes(&self) -> Vec<KeyState> {
		self.input_batcher.current_state.borrow().unhandled_changes.keyboard_changes.clone()
	}

	pub fn get_mouse_button_changes(&self) -> Vec<MouseButtonState> {
		self.input_batcher.current_state.borrow().unhandled_changes.mouse_button_changes.clone()
	}

	pub fn get_scroll_changes(&self) -> Option<Scroll> {
		self.input_batcher.current_state.borrow().unhandled_changes.scroll_changes.clone()
	}

	pub fn get_touch_state_changes(&self) -> HashMap<i32, TouchState> {
		self.input_batcher.current_state.borrow().unhandled_changes.touch_state_changes.clone()
	}

	pub fn get_touch_movement(&self) -> HashMap<i32, TouchMovement> {
		self.input_batcher.current_state.borrow().unhandled_changes.touch_movement.clone()
	}
}

impl Drop for BatchedInputHandler<'_> {
	fn drop(&mut self) { self.input_batcher.store_last(); }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ActiveInput {
	Keyboard(KeyCode),
	MouseButton(MouseButton),
	Touch(i32),
}

#[derive(Clone, Debug)]
pub struct InputState {
	current_set: HashSet<ActiveInput>,
	mouse_position: Option<LogicalCursorPosition>,
	touch_position: HashMap<i32, LogicalCursorPosition>,
	unhandled_changes: FrameDifferences,
}

impl InputState {
	fn new() -> Self {
		Self {
			current_set: Default::default(),
			mouse_position: None,
			touch_position: Default::default(),
			unhandled_changes: FrameDifferences::default(),
		}
	}

	pub fn is_key_active(&self, key: KeyCode) -> bool {
		self.current_set.contains(&ActiveInput::Keyboard(key))
	}

	pub fn is_mouse_button_active(&self, button: MouseButton) -> bool {
		self.current_set.contains(&ActiveInput::MouseButton(button))
	}

	#[allow(dead_code)]
	pub fn current_mouse_location(&self) -> Option<LogicalCursorPosition> { self.mouse_position }

	pub fn active_touch_identifiers(&self) -> Vec<i32> {
		self.current_set.iter().filter_map(|input| {
			match input {
				ActiveInput::Touch(i) => Some(*i),
				_ => None,
			}
		}).collect()
	}

	pub fn current_touch_position(&self, identifier: i32) -> Option<LogicalCursorPosition> {
		self.touch_position.get(&identifier).cloned()
	}

}

type KeyCode = String;

type MouseEventHandler = Closure<dyn FnMut(MouseEvent)>;
type MouseButtonHandler = Closure<dyn FnMut(MouseEvent)>;
type MouseScrollHandler = Closure<dyn FnMut(WheelEvent)>;
type KeyboardEventHandler = Closure<dyn FnMut(KeyboardEvent)>;
type TouchEventHandler = Closure<dyn FnMut(TouchEvent)>;

#[allow(dead_code)]
struct MouseButtonHandlers {
	down: MouseButtonHandler,
	up: MouseButtonHandler,
}

#[allow(dead_code)]
struct KeyboardEventHandlers {
	down: KeyboardEventHandler,
	up: KeyboardEventHandler,
}

type TouchPositionHandler = TouchEventHandler;

struct TouchStateHandlers {
	touch_start: TouchEventHandler,
	touch_end: TouchEventHandler,
	touch_cancel: TouchEventHandler,
}

trait IncorporateState
where
	Self: Clone,
{
	fn incorporate(&self, new_input: UserInput<KeyCode>) -> Self;
}

pub type MouseMovement = LogicalCursorPosition;

#[derive(Clone, Debug, Default)]
pub struct TouchMovement {
	pub identifier: i32,
	pub difference: LogicalCursorPosition,
}

#[derive(Clone, Debug, Default)]
struct FrameDifferences {
	keyboard_changes: Vec<KeyState>,
	mouse_button_changes: Vec<MouseButtonState>,
	mouse_movement: Option<MouseMovement>,
	scroll_changes: Option<Scroll>,
	touch_movement: HashMap<i32, TouchMovement>,
	touch_state_changes: HashMap<i32, TouchState>,
}

impl FrameDifferences {
	fn clear(&mut self) {
		self.keyboard_changes.clear();
		self.mouse_button_changes.clear();
		self.mouse_movement = None;
		self.scroll_changes = None;
		self.touch_movement.clear();
		self.touch_state_changes.clear();
	}
}

impl IncorporateState for InputState {
	fn incorporate(&self, new_input: UserInput<KeyCode>) -> Self {
		let mut new_state = self.clone();

		match new_input {
			UserInput::Keyboard(KeyState { key, state: SwitchState::Pressed }) => {
				let is_new = new_state.current_set.insert(ActiveInput::Keyboard(key.clone()));
				if is_new {
					new_state
						.unhandled_changes
						.keyboard_changes
						.push(KeyState { key, state: SwitchState::Pressed });
				}
			}
			UserInput::Keyboard(KeyState { key, state: SwitchState::Released }) => {
				let was_removed = new_state.current_set.remove(&ActiveInput::Keyboard(key.clone()));
				if was_removed {
					new_state
						.unhandled_changes
						.keyboard_changes
						.push(KeyState { key, state: SwitchState::Released });
				}
			}
			UserInput::MouseButton(MouseButtonState { button, state: SwitchState::Pressed }) => {
				let is_new = new_state.current_set.insert(ActiveInput::MouseButton(button.clone()));
				if is_new {
					new_state
						.unhandled_changes
						.mouse_button_changes
						.push(MouseButtonState { button, state: SwitchState::Pressed });
				}
			}
			UserInput::MouseButton(MouseButtonState { button, state: SwitchState::Released }) => {
				let was_removed =
					new_state.current_set.remove(&ActiveInput::MouseButton(button.clone()));
				if was_removed {
					new_state
						.unhandled_changes
						.mouse_button_changes
						.push(MouseButtonState { button, state: SwitchState::Released });
				}
			}
			UserInput::CursorPosition(new_position) => {
				let previous_position = new_state.mouse_position;

				new_state.mouse_position = Some(new_position.clone());

				if previous_position.is_some() {
					new_state.unhandled_changes.mouse_movement =
						Some(new_position - previous_position.unwrap());
				}
			}
			UserInput::TouchPosition(LogicalTouchPosition { identifier, position }) => {
				let previous_position = new_state.touch_position.get(&identifier).cloned();

				new_state.touch_position.insert(identifier, position.clone());

				if previous_position.is_some() {
					let difference = position - previous_position.unwrap();
					new_state
						.unhandled_changes
						.touch_movement
						.insert(identifier, TouchMovement { identifier, difference });
				}
			}
			UserInput::Scroll(scroll) => {
				new_state.unhandled_changes.scroll_changes = Some(scroll);
			}
			UserInput::Touch(TouchState { identifier, state: SwitchState::Pressed }) => {
				let is_new = new_state.current_set.insert(ActiveInput::Touch(identifier));
				if is_new {
					new_state
						.unhandled_changes
						.touch_state_changes
						.insert(identifier, TouchState { identifier, state: SwitchState::Pressed });
				}
			}
			UserInput::Touch(TouchState { identifier, state: SwitchState::Released }) => {
				let was_removed = new_state.current_set.remove(&ActiveInput::Touch(identifier));
				if was_removed {
					new_state.unhandled_changes.touch_state_changes.insert(
						identifier,
						TouchState { identifier, state: SwitchState::Released },
					);

					// Don't store the old position, since this ID is no longer touching
					if let None = new_state.touch_position.remove(&identifier) {
						ghg_log!("Error removing touch position {identifier} from state!");
					}

				}
			}
			// UserInput::FocusChange(is_focused) => {
			//     ghg_log!("Focus changed: is_focused = {}", is_focused);
			//     new_state.mouse_position = None;
			// }
			#[allow(unreachable_patterns)]
			unhandled => {
				ghg_log!("Unhandled input: {:?}", unhandled);
			}
		}

		new_state
	}
}

fn add_mouse_move_handler(
	canvas: &HtmlCanvasElement,
	current_state: Rc<RefCell<InputState>>,
) -> MouseEventHandler {
	let mouse_move_event_handler = Closure::wrap(Box::new(move |e: MouseEvent| {
		let new_state =
			UserInput::<KeyCode>::CursorPosition(nglm::vec2(e.screen_x(), e.screen_y()));

		current_state.replace_with(move |previous| previous.incorporate(new_state));
	}) as Box<dyn FnMut(MouseEvent)>);

	canvas.set_onmousemove(Some(mouse_move_event_handler.as_ref().unchecked_ref()));
	mouse_move_event_handler
}

fn add_mouse_button_handlers(
	canvas: &HtmlCanvasElement,
	current_state: &Rc<RefCell<InputState>>,
) -> MouseButtonHandlers {
	let mouse_down_event_handler =
		make_mouse_button_handler(current_state.clone(), SwitchState::Pressed);
	canvas.set_onmousedown(Some(mouse_down_event_handler.as_ref().unchecked_ref()));

	let mouse_up_event_handler =
		make_mouse_button_handler(current_state.clone(), SwitchState::Released);
	canvas.set_onmouseup(Some(mouse_up_event_handler.as_ref().unchecked_ref()));

	MouseButtonHandlers { down: mouse_down_event_handler, up: mouse_up_event_handler }
}

fn make_mouse_button_handler(
	current_state: Rc<RefCell<InputState>>,
	switch_state: SwitchState,
) -> MouseButtonHandler {
	Closure::wrap(Box::new(move |e: MouseEvent| {
		let button_index = e.button();

		let button = match button_index {
			0 => MouseButton::Left,
			2 => MouseButton::Right,
			1 => MouseButton::Middle,
			x => MouseButton::Extra(x as u16),
		};

		let new_state =
			UserInput::<KeyCode>::MouseButton(MouseButtonState { button, state: switch_state });

		current_state.replace_with(move |previous| previous.incorporate(new_state));
	}) as Box<dyn FnMut(MouseEvent)>)
}

fn add_mouse_scroll_handler(
	canvas: &HtmlCanvasElement,
	current_state: Rc<RefCell<InputState>>,
) -> MouseScrollHandler {
	let mouse_scroll_handler = Closure::wrap(Box::new(move |e: WheelEvent| {
		let new_state = UserInput::<KeyCode>::Scroll(Scroll::new(
			e.delta_x() as f32,
			e.delta_y() as f32,
			e.delta_z() as f32,
		));
		current_state.replace_with(move |previous| previous.incorporate(new_state));
	}) as Box<dyn FnMut(WheelEvent)>);

	canvas.set_onwheel(Some(mouse_scroll_handler.as_ref().unchecked_ref()));
	mouse_scroll_handler
}

fn add_keyboard_handlers(
	canvas: &HtmlCanvasElement,
	current_state: &Rc<RefCell<InputState>>,
) -> KeyboardEventHandlers {
	let key_down_event_handler = make_key_handler(current_state.clone(), SwitchState::Pressed);
	canvas.set_onkeydown(Some(key_down_event_handler.as_ref().unchecked_ref()));

	let key_up_event_handler = make_key_handler(current_state.clone(), SwitchState::Released);
	canvas.set_onkeyup(Some(key_up_event_handler.as_ref().unchecked_ref()));

	KeyboardEventHandlers { down: key_down_event_handler, up: key_up_event_handler }
}

fn make_key_handler(
	current_state: Rc<RefCell<InputState>>,
	switch_state: SwitchState,
) -> KeyboardEventHandler {
	Closure::wrap(Box::new(move |e: KeyboardEvent| {
		if !e.repeat() {
			let new_state =
				UserInput::<KeyCode>::Keyboard(KeyState { key: e.code(), state: switch_state });

			current_state.replace_with(move |previous| previous.incorporate(new_state));
		}
	}) as Box<dyn FnMut(KeyboardEvent)>)
}

fn add_touch_position_handler(
	canvas: &HtmlCanvasElement,
	current_state: Rc<RefCell<InputState>>,
) -> TouchPositionHandler {
	let handler = Closure::wrap(Box::new(move |e: TouchEvent| {
		e.prevent_default();

		let touches = e.changed_touches();
		for i in 0..touches.length() {
			let touch = touches.get(i).expect(format!("Invalid touch index {i}").as_str());
			let position = nglm::vec2(touch.screen_x(), touch.screen_y());
			let new_state = UserInput::<KeyCode>::TouchPosition(LogicalTouchPosition {
				identifier: touch.identifier(),
				position,
			});
			current_state.replace_with(move |previous| previous.incorporate(new_state));
		}
	}) as Box<dyn FnMut(TouchEvent)>);

	canvas.set_ontouchmove(Some(handler.as_ref().unchecked_ref()));

	handler
}

fn add_touch_state_handlers(
	canvas: &HtmlCanvasElement,
	current_state: &Rc<RefCell<InputState>>,
) -> TouchStateHandlers {
	let touch_start_handler = make_touch_state_handler(current_state.clone(), SwitchState::Pressed);
	let touch_end_handler = make_touch_state_handler(current_state.clone(), SwitchState::Released);
	let touch_cancel_handler =
		make_touch_state_handler(current_state.clone(), SwitchState::Released);

	canvas.set_ontouchstart(Some(touch_start_handler.as_ref().unchecked_ref()));
	canvas.set_ontouchend(Some(touch_end_handler.as_ref().unchecked_ref()));
	canvas.set_ontouchcancel(Some(touch_cancel_handler.as_ref().unchecked_ref()));

	TouchStateHandlers {
		touch_start: touch_start_handler,
		touch_end: touch_end_handler,
		touch_cancel: touch_cancel_handler,
	}
}

fn make_touch_state_handler(
	current_state: Rc<RefCell<InputState>>,
	switch_state: SwitchState,
) -> TouchEventHandler {
	Closure::wrap(Box::new(move |e: TouchEvent| {
		e.prevent_default();

		let touches = e.changed_touches();
		for i in 0..touches.length() {
			let touch = touches.get(i).expect(format!("Invalid touch index {i}").as_str());
			let new_state = UserInput::<KeyCode>::Touch(TouchState {
				identifier: touch.identifier(),
				state: switch_state,
			});
			current_state.replace_with(move |previous| previous.incorporate(new_state));
		}
	}) as Box<dyn FnMut(TouchEvent)>)
}

// For debug only
impl Drop for InputBatcher {
	fn drop(&mut self) {
		ghg_log!("Uh oh! InputHandler was dropped!");
	}
}
