use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Response};
use crate::render_core::canvas::window;

pub async fn fetch_blob(url: &str) -> Result<Blob, JsValue> {
    let resp_value = JsFuture::from(window().fetch_with_str(url)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let response: Response = resp_value.dyn_into().unwrap();

    let blob = JsFuture::from(response.blob()?).await?;
    assert!(blob.is_instance_of::<Blob>());

    Ok(blob.dyn_into().unwrap())
}
