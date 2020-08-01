use anyhow::*;
use http;
use serde::de;
use web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use apilib::*;

pub async fn fetch<T: de::DeserializeOwned>(request : http::Request<String>) -> anyhow::Result<http::Response<T>> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(request.method().as_str());
    opts.mode(web_sys::RequestMode::Cors);
    opts.body(Some(&JsValue::from_str(request.body().as_str())));

    let url = request.uri().to_string();

    let request = match web_sys::Request::new_with_str_and_init(&url, &opts) {
        Ok(request) => Ok(request),
        Err(js_value) => Err(anyhow!("Failed to create web_sys request {:?}!", js_value))
    }?;

    match request
        .headers()
        .set("Accept", "application/json") {
        Ok(()) => Ok(()),
        Err(js_value) => Err(anyhow!("Failed to add accept header!"))
    }?;

    let window = web_sys::window().unwrap();

    let response = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(response) => Ok(response),
        Err(js_value) => Err(anyhow!("Failed to execute request!")),
    }?;

    assert!(response.is_instance_of::<web_sys::Response>());
    let response: web_sys::Response = response.dyn_into().unwrap();

    let json_response = match response.json() {
        Ok(response) => Ok(response),
        Err(js_value) => Err(anyhow!("Failed to get json response!"))
    }?;

    // Convert this other `Promise` into a rust `Future`.
    let json = match JsFuture::from(json_response).await {
        Ok(value) => Ok(value),
        Err(js_error) => Err(anyhow!("Failed to await js_future!"))
    }?;

    // Use serde to parse the JSON into a struct.
    let branch_info: T = json.into_serde().unwrap();

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(branch_info)?;

    return Ok(response);
}

pub async fn new_session_id() -> anyhow::Result<http::Response<String>> {
    let mut opts = web_sys::RequestInit::new();
    opts.method("GET");
    opts.mode(web_sys::RequestMode::Cors);

    let url = "http://rusty-games.localhost/api/session-id/new";

    let request = match web_sys::Request::new_with_str_and_init(&url, &opts) {
        Ok(request) => Ok(request),
        Err(js_value) => Err(anyhow!("Failed to create web_sys request!"))
    }?;

    match request
        .headers()
        .set("Accept", "application/json") {
        Ok(()) => Ok(()),
        Err(js_value) => Err(anyhow!("Failed to add accept header!"))
    }?;

    let window = web_sys::window().unwrap();

    let response = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(response) => Ok(response),
        Err(js_value) => Err(anyhow!("Failed to execute request!")),
    }?;

    assert!(response.is_instance_of::<web_sys::Response>());
    let response: web_sys::Response = response.dyn_into().unwrap();

    let json_response = match response.json() {
        Ok(response) => Ok(response),
        Err(js_value) => Err(anyhow!("Failed to get json response!"))
    }?;

    // Convert this other `Promise` into a rust `Future`.
    let json = match JsFuture::from(json_response).await {
        Ok(value) => Ok(value),
        Err(js_error) => Err(anyhow!("Failed to await js_future!"))
    }?;

    // Use serde to parse the JSON into a struct.
    let branch_info: String = json.into_serde().unwrap();

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(branch_info)?;

    return Ok(response);
}

pub async fn list_scores_http(request : &ListScoresRequest) -> anyhow::Result<http::Response<Vec<PlayerScore>>> {
    let request = serde_json::to_string(&request)?;
    let uri = "http://rusty-games.localhost/api/score/list";

    let request = http::Request::builder()
        .uri(uri)
        .method("POST")
        .body(request)?;

    let response : http::Response<Vec<PlayerScore>> = fetch(request).await?;

    return Ok(response);
}
