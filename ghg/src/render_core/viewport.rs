use std::cell::RefCell;

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::render_core;

pub struct Viewport {
	canvas: HtmlCanvasElement,
	context: WebGl2RenderingContext,
	width: RefCell<f32>,
	height: RefCell<f32>,
}

impl Viewport {
	pub fn new(canvas: HtmlCanvasElement, context: WebGl2RenderingContext) -> Self {
		let (uwidth, uheight) = render_core::canvas::update_canvas_size(&canvas);

		Self {
			canvas,
			context,
			width: RefCell::new(uwidth as f32),
			height: RefCell::new(uheight as f32),
		}
	}

	// pub fn aspect_ratio(&self) -> f32 {
	//     return self.width() / self.height();
	// }

	pub fn on_frame(&self) {
		let (uwidth, uheight) = render_core::canvas::update_canvas_size(&self.canvas);

		let width = uwidth as f32;
		let height = uheight as f32;

		if width != self.width() || height != self.height() {
			self.width.replace(width);
			self.height.replace(height);
		}

		self.context.viewport(0, 0, width.round() as i32, height.round() as i32);
	}

	pub fn context(&self) -> &WebGl2RenderingContext { &self.context }

	pub fn width(&self) -> f32 { self.width.borrow().clone() }

	pub fn height(&self) -> f32 { self.height.borrow().clone() }
}
