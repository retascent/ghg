use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::Duration;
use crate::utils::prelude::*;

use wasm_bindgen::JsCast;
use crate::Viewport;

pub type AnimationFn = Box<dyn FnMut(&Viewport, Duration)>;

pub fn wrap_animation_body<F: 'static + FnMut(&Viewport, Duration)>(f: F) -> AnimationFn {
    Box::new(f)
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn run_animation_loop(viewport: Viewport, mut animation_body: AnimationFn)
{
    let next_frame = Rc::new(RefCell::new(None));
    let start_frame = next_frame.clone();

    let performance = window()
        .performance()
        .expect("performance should be available");

    let last_frame_time = RefCell::new(performance.now());

    *start_frame.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let this_frame_time = performance.now();
        let duration_millis: f64 = this_frame_time - last_frame_time.borrow().clone();
        let duration = Duration::from_micros((duration_millis * 1000.0) as u64);
        last_frame_time.replace(this_frame_time);

        viewport.on_frame();
        animation_body.deref_mut()(&viewport, duration);
        request_animation_frame(next_frame.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(start_frame.borrow().as_ref().unwrap());
}
