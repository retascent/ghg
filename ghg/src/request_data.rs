use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Response};

use crate::render_core::canvas::window;

pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
	let blob = fetch_blob(url).await?;
	let bytes = blob_to_bytes(blob).await?;
	Ok(bytes)
}

pub async fn fetch_blob(url: &str) -> Result<Blob, JsValue> {
	let resp_value = JsFuture::from(window().fetch_with_str(url)).await?;
	assert!(resp_value.is_instance_of::<Response>());
	let response: Response = resp_value.dyn_into().unwrap();

	let blob = JsFuture::from(response.blob()?).await?;
	assert!(blob.is_instance_of::<Blob>());

	Ok(blob.dyn_into().unwrap())
}

async fn blob_to_bytes(blob: Blob) -> Result<Vec<u8>, JsValue> {
	let array_buffer_promise: JsFuture = blob.array_buffer().into();

	let array_buffer: JsValue = array_buffer_promise.await?;

	Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}
