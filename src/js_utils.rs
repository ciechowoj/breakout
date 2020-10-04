use wasm_bindgen::prelude::*;

trait JsValueError<T> {
    fn to_result(self) -> std::result::Result<T, String>;
}

impl<T> JsValueError<T> for std::result::Result<T, wasm_bindgen::JsValue> {
    fn to_result(self) -> std::result::Result<T, String> {
        return match self {
            Ok(value) => Ok(value),
            Err(error) => Err(format!("{:?}", error))
        };
    }
}

trait JsValueEx {
    fn try_get_property(&self, id : &'static str) -> std::result::Result<JsValue, String>;
    fn try_get_property_f64(&self, id : &'static str) -> std::result::Result<f64, String>;

    fn get_property(&self, id : &'static str) -> JsValue { self.try_get_property(id).unwrap() }
    fn get_property_f64(&self, id : &'static str) -> f64 { self.try_get_property_f64(id).unwrap() }
}

impl JsValueEx for JsValue {
    fn try_get_property(&self, id : &'static str) -> std::result::Result<JsValue, String> {
        let property = js_sys::Reflect::get(&self, &JsValue::from_str(id)).to_result()?;
        return Ok(property);
    }

    fn try_get_property_f64(&self, id : &'static str) -> std::result::Result<f64, String> {
        return match self.try_get_property(id)?.as_f64() {
            Some(value) => Ok(value),
            None => Err(format!("Cannot convert the '{}' property to f64!", id))
        };
    }
}
