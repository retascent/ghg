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

use std::ops::Deref;
use prelude::*;

#[wasm_bindgen]
pub fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

pub fn assign_shared<T>(f: &std::rc::Rc<std::cell::RefCell<T>>, value: T) {
    *f.borrow_mut() = value;
}

pub fn read_shared<T: Copy>(f: &std::rc::Rc<std::cell::RefCell<T>>) -> T {
    *f.borrow().deref()
}

macro_rules! ghg_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[allow(unused_imports)]
pub(crate) use ghg_log;

macro_rules! ghg_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

#[allow(unused_imports)]
pub(crate) use ghg_error;

#[macro_export]
macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

#[macro_export]
macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}

#[allow(unused_imports)]
pub use clone_all;

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + ghg::count!($($xs)*));
}

#[allow(unused_imports)]
pub use count;
