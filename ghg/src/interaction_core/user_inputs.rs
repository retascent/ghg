#![allow(dead_code)]

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SwitchState {
	Pressed,
	Released,
}

// TODO: Incomplete
// #[derive(Clone, Copy, Debug)]
// pub enum VirtualKey {
//     Unknown, /// Only exists because this is incomplete
//
//     _1,
//     _2,
//     _3,
//     _4,
//     _5,
//     _6,
//     _7,
//     _8,
//     _9,
//     _0,
//
//     A,
//     B,
//     C,
//     D,
//     E,
//     F,
//     G,
//     H,
//     I,
//     J,
//     K,
//     L,
//     M,
//     N,
//     O,
//     P,
//     Q,
//     R,
//     S,
//     T,
//     U,
//     V,
//     W,
//     X,
//     Y,
//     Z,
//
//     Escape,
//
//     Space,
//
//     LeftShift,
//     LeftControl,
//     LeftAlt,
//     LeftMod, // e.g. the Windows key
//
//     RightShift,
//     RightControl,
//     RightAlt,
//     RightMod, // e.g. the Windows key
// }

#[derive(Clone, Debug)]
pub struct KeyCodeState<T: Clone> {
	pub key: T,
	pub state: SwitchState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Extra(u16),
}

#[derive(Clone, Debug)]
pub struct MouseButtonState {
	pub button: MouseButton,
	pub state: SwitchState,
}

#[derive(Clone, Debug)]
pub struct Scroll {
	pub delta_x: f32,
	pub delta_y: f32,
	pub delta_z: f32,
}

impl Scroll {
	pub fn new(delta_x: f32, delta_y: f32, delta_z: f32) -> Self {
		Self { delta_x, delta_y, delta_z }
	}
}

pub type LogicalCursorPosition = nglm::I32Vec2;

#[derive(Clone, Debug)]
pub struct LogicalTouchPosition {
	pub identifier: i32,
	pub position: LogicalCursorPosition,
}

#[derive(Clone, Debug)]
pub struct TouchState {
	pub identifier: i32,
	pub state: SwitchState,
}

#[derive(Clone, Debug)]
pub enum UserInput<T: Clone> {
	Keyboard(KeyCodeState<T>),
	MouseButton(MouseButtonState),
	CursorPosition(LogicalCursorPosition),
	Scroll(Scroll),
	TouchPosition(LogicalTouchPosition),
	Touch(TouchState),
}
