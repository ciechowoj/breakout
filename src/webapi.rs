use anyhow::*;
use http;
use serde::{Deserialize, Serialize};
use web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;


// match (request.method().as_str(), request.uri().path()) {
//     ("GET", "/api/session-id/new") => { print_output(&new_session_id().await?)?; return Ok(()); },
//     _ => ()
// };


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

// pub async fn new_session_id() -> Result<JsValue, JsValue> {
//     let mut opts = RequestInit::new();
//     opts.method("GET");
//     opts.mode(RequestMode::Cors);

//     let url = format!("https://api.github.com/repos/{}/branches/master", repo);

//     let request = Request::new_with_str_and_init(&url, &opts)?;

//     request
//         .headers()
//         .set("Accept", "application/vnd.github.v3+json")?;

//     let window = web_sys::window().unwrap();
//     let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

//     // `resp_value` is a `Response` object.
//     assert!(resp_value.is_instance_of::<Response>());
//     let resp: Response = resp_value.dyn_into().unwrap();

//     // Convert this other `Promise` into a rust `Future`.
//     let json = JsFuture::from(resp.json()?).await?;

//     // Use serde to parse the JSON into a struct.
//     let branch_info: Branch = json.into_serde().unwrap();

//     // Send the `Branch` struct back to JS as an `Object`.
//     Ok(JsValue::from_serde(&branch_info).unwrap())
// }
