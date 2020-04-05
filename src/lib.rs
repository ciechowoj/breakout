mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use web_sys::*;

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
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    document.set_title("Omg! It works!");

    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    body.append_child(&canvas).ok();

    struct Recursive {
        value: Rc<dyn Fn(Recursive)>
    };

    let update = move |update: Recursive| {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
    
        let inner : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            // context.set_fill_style(&JsValue::from_str("red"));
            context.clear_rect(0.0, 0.0, 200.0, 200.0);

            context.set_fill_style(&JsValue::from_str("black"));
            if let Some(value) = js_value.as_f64() {
                context.fill_rect(100.0 * (value * 0.001).fract(), 10.0, 100.0, 100.0);
            }
            else {
                context.fill_rect(100.0, 10.0, 100.0, 100.0);
            }

            (update.value)(Recursive { value: update.value.clone() });
        });

        let closure = Closure::once_into_js(inner as Box<dyn FnMut(JsValue)>);
    
        window.request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
    };

    let update_clone = Rc::new(update);
    update_clone(Recursive { value: update_clone.clone() });
}
