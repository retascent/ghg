#![feature(extern_types)]
extern crate nalgebra_glm as nglm;

use web_sys::WebGl2RenderingContext;
use utils::prelude::*;

#[macro_use]
mod utils;
mod render_core;
mod interaction_core;
mod data_core;
mod application;

use crate::render_core::animation::run_animation_loop;
use crate::render_core::viewport::Viewport;
use crate::utils::set_panic_hook;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    set_panic_hook();
    let (canvas, context) = render_core::canvas::get_webgl2_canvas().ok_or("Failed to create WebGL2 context")?;

    // Workaround: https://stackoverflow.com/a/18934718/1403459
    canvas.set_attribute("tabindex", "0")?;
    canvas.focus()?;

    context.enable(WebGl2RenderingContext::DEPTH_TEST);
    context.depth_func(WebGl2RenderingContext::LESS);

    let viewport = Viewport::new(canvas.clone(), context.clone());
    let animation_body = application::animation_loop::get_animation_loop(canvas, context)?;
    run_animation_loop(viewport, animation_body); // Never returns

    Ok(())
}
