use utils::prelude::*;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::utils;

pub fn get_webgl2_canvas() -> Option<(HtmlCanvasElement, WebGl2RenderingContext)> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("render_canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().ok()?;

    let context: WebGl2RenderingContext = canvas
        .get_context("webgl2").ok()?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>().ok()?;

    Some((canvas, context))
}

pub fn update_canvas_size(canvas: &HtmlCanvasElement) -> (u32, u32) {
    let dpr: f64 = window().device_pixel_ratio();
    let display_width: u32 = (canvas.client_width() as f64 * dpr).round() as u32;
    let display_height: u32 = (canvas.client_height() as f64 * dpr).round() as u32;

    let need_resize = canvas.width()  != display_width ||
        canvas.height() != display_height;

    if need_resize {
        ghg_log!("Resizing canvas: {} x {} -> {} x {}",
            canvas.width(), canvas.height(), display_width, display_height);

        canvas.set_width(display_width);
        canvas.set_height(display_height);
    }

    (display_width, display_height)
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}
