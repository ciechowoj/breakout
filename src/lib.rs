mod utils;

use std::any::Any;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use web_sys::*;
use utils::set_panic_hook;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let html = document.document_element().expect("document should have a html");
    let html = html.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();

    html.style().set_property("height", "100%").unwrap();

    let body = document.body().expect("document should have a body");
    document.set_title("Omg! It works!");

    body.style().set_property("margin", "0px").unwrap();
    body.style().set_property("padding", "0px").unwrap();
    body.style().set_property("width", "100%").unwrap();
    body.style().set_property("height", "100%").unwrap();

    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_width(640);
    canvas.set_height(480);

    canvas.style().set_property("border", "none").unwrap();
    canvas.style().set_property("min-width", "800px").unwrap();
    canvas.style().set_property("height", "100%").unwrap();
    canvas.style().set_property("margin-left", "auto").unwrap();
    canvas.style().set_property("margin-right", "auto").unwrap();
    canvas.style().set_property("padding-left", "0px").unwrap();
    canvas.style().set_property("padding-right", "0px").unwrap();
    canvas.style().set_property("display", "block").unwrap();

    body.append_child(&canvas).ok();

    let context : String = "1".to_string();

    struct Recursive {
        value: Rc<dyn Fn(Rc<Recursive>)>,
        context: RefCell<Box<dyn Any>>
    };

    let update = move |update: Rc<Recursive>| {
        let rendering_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
    
        let inner : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            if let Some(value) = js_value.as_f64() {
                crate::update(&mut **update.context.borrow_mut(), &rendering_context, value);
            }

            let update_clone = update.clone();

            (update.value)(update_clone);
        });

        let closure = Closure::once_into_js(inner as Box<dyn FnMut(JsValue)>);
    
        window.request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
    };

    let update_clone = Rc::new(update);

    update_clone(Rc::new(Recursive { 
        value: update_clone.clone(),
        context: RefCell::new(Box::new(context))
    }));
}


pub fn update(context : &mut dyn Any, rendering_context : &CanvasRenderingContext2d, time : f64) {
    let context = context.downcast_mut::<String>();

    if context.is_some() {
        let context = context.unwrap();

        /*web_sys::console::log_1(&JsValue::from(context.to_string()));
        let parsed = (context.to_string().parse::<i32>().unwrap() + 1) % 10;

        context.clear();
        context.insert_str(0, &parsed.to_string());*/
    }
    



    let canvas = rendering_context.canvas().unwrap();

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    rendering_context.clear_rect(0.0, 0.0, width, height);
    rendering_context.set_fill_style(&JsValue::from_str("black"));
    rendering_context.fill_rect((width - 100.0) * (time * 0.0001).fract(), 10.0, 100.0, 100.0);

    rendering_context.fill_rect(0.0, 200.0, width, 100.0);
}