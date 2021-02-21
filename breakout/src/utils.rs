use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

pub enum ClosureHandle {
    Empty,
    Handle {
        closure : Box<dyn std::any::Any>,
        js_function : js_sys::Function
    }
}

impl ClosureHandle {
    pub fn new<'a, Args : wasm_bindgen::convert::FromWasmAbi + 'static, Result : wasm_bindgen::convert::IntoWasmAbi + 'static>(closure : Box<dyn FnMut(Args) -> Result>) -> ClosureHandle {
        let closure = Closure::wrap(closure);
        let js_function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();

        ClosureHandle::Handle {
            closure: Box::new(closure),
            js_function: js_function
        }
    }

    pub fn function(&self) -> &js_sys::Function {
        match self {
            Self::Handle { closure: _, js_function } => {
                return &js_function;
            }
            Self::Empty => {
                panic!();
            }
        }
    }
}
