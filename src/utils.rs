use std::num::*;
use wasm_bindgen::prelude::*;
use serde_json;

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

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Msg(&'static str),
    Str(String),
    Js(JsValue)
}

impl From<JsValue> for Error {
    fn from(js_value: JsValue) -> Self {
        Error::Js(js_value)
    }
}

impl From<strum::ParseError> for Error {
    fn from(error: strum::ParseError) -> Self {
        Error::Str(error.to_string())
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::Str(error.to_string())
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Error::Str(error.to_string())
    }
}

pub type ExpectedUnit = std::result::Result<(), Error>;
pub type Expected<T> = std::result::Result<T, Error>;
