
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

pub trait JsValueError<T> {
    fn to_anyhow(self) -> anyhow::Result<T>;
}

impl<T> JsValueError<T> for std::result::Result<T, wasm_bindgen::JsValue> {
    fn to_anyhow(self) -> anyhow::Result<T> {
        return match self {
            Ok(value) => Ok(value),
            Err(error) => Err(anyhow::anyhow!("{:?}", error))
        };
    }
}
