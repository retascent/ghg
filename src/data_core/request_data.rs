use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Request, RequestInit, RequestMode, Response};
use crate::render_core::canvas::window;

pub async fn fetch_blob(url: &str) -> Result<Blob, JsValue> {
    // let mut opts = RequestInit::new();
    // opts.method("GET");
    // opts.mode(RequestMode::SameOrigin);
    // let request = Request::new_with_str_and_init(&url.to_owned(), &opts)?;
    // let resp_value = JsFuture::from(window().fetch_with_request(&request)).await?;

    let resp_value = JsFuture::from(window().fetch_with_str(url)).await?;
    assert!(resp_value.is_instance_of::<Response>());
    let response: Response = resp_value.dyn_into().unwrap();

    let blob = JsFuture::from(response.blob()?).await?;
    assert!(blob.is_instance_of::<Blob>());

    Ok(blob.dyn_into().unwrap())
}
