use js_sys::Promise;
use wasm_bindgen::closure::{Closure, IntoWasmClosure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, Response};
use crate::render_core::canvas::window;

use crate::utils::prelude::*;

#[derive(Default)]
pub struct GetterContext {
    promise: Option<Promise>,
    on_success: Option<Closure<dyn FnMut(JsValue)>>,
    on_unwrap_blob: Option<Closure<dyn FnMut(Blob)>>,
}

pub async fn fetch_blob(url: &str) -> Result<Blob, JsValue> {
    let resp_value = JsFuture::from(window().fetch_with_str(url)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let response: Response = resp_value.dyn_into().unwrap();
    let blob = JsFuture::from(response.blob()).await?;
    Ok(blob)
}
