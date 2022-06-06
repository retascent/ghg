#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub mod prelude {
    pub use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        // Use `js_namespace` here to bind `console.log(..)` instead of just
        // `log(..)`
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);

        // The `console.log` is quite polymorphic, so we can bind it with multiple
        // signatures. Note that we need to use `js_name` to ensure we always call
        // `log` in JS.
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_u32(a: u32);

        // Multiple arguments too!
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        pub fn log_many(a: &str, b: &str);

        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);
    }
}

use prelude::*;

#[wasm_bindgen]
pub fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[wasm_bindgen]
pub fn read_img(ptr: *mut u8, len: usize) {
    let img = unsafe { Vec::from_raw_parts(ptr, len, len) };

    if let Err(e) = image::load_from_memory(&img) {
        ghg_log!("{}", &e.to_string());
    }
}

macro_rules! ghg_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! ghg_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

#[allow(unused_imports)]
pub(crate) use ghg_log;

#[allow(unused_imports)]
pub(crate) use ghg_error;
