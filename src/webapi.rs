use anyhow::*;
use http;
use serde::de;
use web_sys;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use apilib::*;
use crate::dom_utils as browser;

pub async fn fetch<T: de::DeserializeOwned>(request : http::Request<Option<String>>) -> anyhow::Result<http::Response<T>> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(request.method().as_str());
    opts.mode(web_sys::RequestMode::Cors);

    if let Some(value) = request.body() {
        opts.body(Some(&JsValue::from_str(value.as_str())));
    }

    let url = request.uri().to_string();

    let request = match web_sys::Request::new_with_str_and_init(&url, &opts) {
        Ok(request) => Ok(request),
        Err(js_value) => Err(anyhow!("Failed to create web_sys request {:?}!", js_value))
    }?;

    match request
        .headers()
        .set("Accept", "application/json") {
        Ok(()) => Ok(()),
        Err(_js_value) => Err(anyhow!("Failed to add accept header!"))
    }?;

    let window = browser::window();

    let response = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(response) => Ok(response),
        Err(_js_value) => Err(anyhow!("Failed to execute request!")),
    }?;

    assert!(response.is_instance_of::<web_sys::Response>());
    let response: web_sys::Response = response.dyn_into().unwrap();

    let json_response = match response.json() {
        Ok(response) => Ok(response),
        Err(_js_value) => Err(anyhow!("Failed to get json response!"))
    }?;

    // Convert this other `Promise` into a rust `Future`.
    let json = match JsFuture::from(json_response).await {
        Ok(value) => Ok(value),
        Err(_js_error) => Err(anyhow!("Failed to await js_future!"))
    }?;

    // Use serde to parse the JSON into a struct.
    let response: T = json.into_serde().unwrap();

    let response = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(response)?;

    return Ok(response);
}

pub fn build_uri(relative : &'static str) -> String {
    let location = browser::window().location();
    return format!("{}//{}/{}", location.protocol().unwrap(), location.hostname().unwrap(), relative);
}

pub async fn new_session_id_http() -> anyhow::Result<http::Response<String>> {
    let uri = build_uri("api/session-id/new");

    let request = http::Request::builder()
        .uri(uri)
        .method("GET")
        .body(None)?;

    let response : http::Response<String> = fetch(request).await?;

    return Ok(response);
}

pub async fn new_session_id() -> anyhow::Result<String> {
    let response = new_session_id_http().await?;

    if response.status() != http::status::StatusCode::OK {
        return Err(anyhow::anyhow!("Failed to get a new session id."));
    }

    let result = response.into_body();
    return Ok(result);
}

pub async fn new_score_http(request : &NewScoreRequest) -> anyhow::Result<http::Response<NewScoreResponse>> {
    let uri = build_uri("api/score/new");
    let request = serde_json::to_string(&request)?;

    let request = http::Request::builder()
        .uri(uri)
        .method("POST")
        .body(Some(request))?;

    let response : http::Response<NewScoreResponse> = fetch(request).await?;

    return Ok(response);
}

pub async fn new_score(request : &NewScoreRequest) -> anyhow::Result<NewScoreResponse> {
    let response = new_score_http(request).await?;

    if response.status() != http::status::StatusCode::OK {
        return Err(anyhow::anyhow!("Failed to create a new score."));
    }

    let result = response.into_body();
    return Ok(result);
}

async fn rename_score_http(request : &RenameScoreRequest) -> anyhow::Result<http::Response<()>> {
    let uri = build_uri("api/score/rename");
    let request = serde_json::to_string(&request)?;

    let request = http::Request::builder()
        .uri(uri)
        .method("POST")
        .body(Some(request))?;

    let response : http::Response<()> = fetch(request).await?;

    return Ok(response);
}

pub async fn rename_score(request : &RenameScoreRequest) -> anyhow::Result<()> {
    let response = rename_score_http(request).await?;

    if response.status() != http::status::StatusCode::OK {
        return Err(anyhow::anyhow!("Failed to rename a score."));
    }

    return Ok(());
}

pub async fn _list_scores_http(request : &ListScoresRequest) -> anyhow::Result<http::Response<Vec<PlayerScore>>> {
    let uri = build_uri("api/score/list");
    let request = serde_json::to_string(&request)?;

    let request = http::Request::builder()
        .uri(uri)
        .method("POST")
        .body(Some(request))?;

    let response : http::Response<Vec<PlayerScore>> = fetch(request).await?;

    return Ok(response);
}
