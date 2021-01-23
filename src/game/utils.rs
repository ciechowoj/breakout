use web_sys::*;
use wasm_bindgen::{JsCast};
use anyhow;

#[derive(Copy, Clone)]
pub struct GameTime {
    pub sim_time : f64,
    pub real_time : f64,
    pub elapsed : f32
}

pub fn create_style_element(document : &Document, sheet : &str, id : &str) -> anyhow::Result<HtmlStyleElement> {
    let style : HtmlStyleElement = document.create_element("style").unwrap().unchecked_into();

    style.set_id(id);
    style.set_type("text/css");
    style.set_inner_html(sheet);

    let head = document.head().unwrap();
    head.append_child(&style).unwrap();

    return Ok(style);
}
