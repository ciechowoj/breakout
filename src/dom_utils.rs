use web_sys::*;
use wasm_bindgen::JsCast;
use crate::utils::*;

pub fn create_html_element(document : &Document, tag : &str, id : &str) -> Expected<HtmlElement> {
    let element = document.create_element(tag)?;
    let element = element.dyn_into::<HtmlElement>()
        .map_err(|_| Error::Msg("Failed to cast 'Element' to 'HtmlElement'."))?;
    element.set_id(id);
    return Ok(element);
}

pub fn create_style_element(document : &Document, sheet : &str, id : &str) -> Expected<HtmlStyleElement> {
    let style = create_html_element(&document, "style", id)?;
    let style = style.dyn_into::<HtmlStyleElement>()
        .map_err(|_| Error::Msg("Failed to cast 'HtmlElement' to 'HtmlStyleElement'."))?;
    style.set_type("text/css");
    style.set_inner_html(sheet);

    let head = document.head().ok_or(Error::Msg("Failed to get 'head' of the document."))?;
    head.append_child(&style)?;

    return Ok(style);
}
