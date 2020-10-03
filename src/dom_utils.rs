use crate::utils::*;
use web_sys::*;
use wasm_bindgen::JsCast;

pub fn window() -> Window {
    return web_sys::window().expect("Failed to retrieve the reference to the global window.");
}

pub fn into_html_element(element : Element) -> HtmlElement {
    let element = element
        .dyn_into::<HtmlElement>()
        .expect("Failed to cast 'Element' to 'HtmlElement'.");

    return element;
}

pub fn create_html_element(document : &Document, tag : &str, id : &str) -> anyhow::Result<HtmlElement> {
    let element = document.create_element(tag).to_anyhow()?;
    let element = element.dyn_into::<HtmlElement>()
        .map_err(|_| anyhow::anyhow!("Failed to cast 'Element' to 'HtmlElement'."))?;
    element.set_id(id);
    return Ok(element);
}

pub fn try_get_html_element_by_id(document : &Document, id : &str) -> anyhow::Result<Option<HtmlElement>> {
    if let Some(element) = document.get_element_by_id(id) {
        let element = element.dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| anyhow::anyhow!("Failed to cast 'Element' to 'HtmlElement'."))?;

        return Ok(Some(element));
    }
    else {
        return Ok(None);
    }
}

pub fn get_html_element_by_id(document : &Document, id : &str) -> anyhow::Result<HtmlElement> {
    if let Some(element) = document.get_element_by_id(id) {
        let element = element.dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| anyhow::anyhow!("Failed to cast 'Element' to 'HtmlElement'."))?;

        return Ok(element);
    }
    else {
        return Err(anyhow::anyhow!("There is no html element with specified id!"));
    }
}

pub fn create_style_element(document : &Document, sheet : &str, id : &str) -> anyhow::Result<HtmlStyleElement> {
    let style = create_html_element(&document, "style", id)?;
    let style = style.dyn_into::<HtmlStyleElement>()
        .map_err(|_| anyhow::anyhow!("Failed to cast 'HtmlElement' to 'HtmlStyleElement'."))?;
    style.set_type("text/css");
    style.set_inner_html(sheet);

    let head = document.head().ok_or(anyhow::anyhow!("Failed to get 'head' of the document."))?;
    head.append_child(&style).to_anyhow()?;

    return Ok(style);
}

