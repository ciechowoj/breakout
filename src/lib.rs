#[macro_use]
mod utils;
mod game;
mod vec2;

use std::any::Any;
use std::cell::RefCell;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use web_sys::*;
use utils::*;
use game::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

fn now_sec(performance : &Performance) -> f64 {
    fn sec_to_ms(x : f64) -> f64 { x * 0.001 };
    return sec_to_ms(performance.now());
}

#[wasm_bindgen]
pub fn greet() {
    fn bind_event_handlers(
        document : &Document,
        input_events : Rc<RefCell<Vec<InputEvent>>>,
        update_struct : Rc<Recursive>) {

        fn get_code(js_value : JsValue) -> Expected<KeyCode> {
            let code = js_sys::Reflect::get(&js_value, &JsValue::from_str("code"))?;
            let code = code.as_string().ok_or(Error::Msg("Expected 'code' field in KeyboardEvent!"))?;
            let code = KeyCode::from_str(&code.to_string())?;
            return Ok(code);
        };

        fn js_to_keydown_event(js_value : JsValue, performance : &Performance) -> Expected<InputEvent> {
            let event = InputEvent::KeyDown { 
                time: now_sec(performance), 
                code: get_code(js_value)?
            };

            return Ok(event);
        }

        fn js_to_keyup_event(js_value : JsValue, performance : &Performance) -> Expected<InputEvent> {
            let event = InputEvent::KeyUp { 
                time: now_sec(performance), 
                code: get_code(js_value)?
            };

            return Ok(event);
        }
    
        let update_struct_clone = update_struct.clone();
        let on_keydown : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_keydown_event(
                js_value,
                &update_struct_clone.performance).unwrap();

                input_events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_keydown as Box<dyn FnMut(JsValue)>);
        document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let update_struct_clone = update_struct.clone();
        let on_keyup : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_keyup_event(
                js_value,
                &update_struct_clone.performance).unwrap();
    
            update_struct_clone.events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_keyup as Box<dyn FnMut(JsValue)>);
        document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    let performance = window.performance().expect("performance should be available");

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

    canvas.set_width(800);
    canvas.set_height(800);

    canvas.style().set_property("border", "none").unwrap();
    canvas.style().set_property("min-width", "800px").unwrap();
    canvas.style().set_property("height", "100%").unwrap();
    canvas.style().set_property("margin-left", "auto").unwrap();
    canvas.style().set_property("margin-right", "auto").unwrap();
    canvas.style().set_property("padding-left", "0px").unwrap();
    canvas.style().set_property("padding-right", "0px").unwrap();
    canvas.style().set_property("display", "block").unwrap();

    body.append_child(&canvas).ok();

    struct Recursive {
        value: Rc<dyn Fn(Rc<Recursive>)>,
        context: RefCell<Box<dyn Any>>,
        events: Rc<RefCell<Vec<InputEvent>>>,
        performance: Performance
    };

    let update = move |update: Rc<Recursive>| {
        let rendering_context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
    
        let inner : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            if let Some(_) = js_value.as_f64() {

                let time = now_sec(&update.performance);

                crate::update(
                    &mut update.context.borrow_mut(),
                    &update.events.borrow(),
                    &rendering_context,
                    time).unwrap();

                update.events.borrow_mut().clear();
            }

            let update_clone = update.clone();

            (update.value)(update_clone);
        });

        let closure = Closure::once_into_js(inner as Box<dyn FnMut(JsValue)>);
    
        window.request_animation_frame(closure.as_ref().unchecked_ref())
            .unwrap();
    };

    let update_clone = Rc::new(update);

    let update_struct = Rc::new(Recursive { 
        value: update_clone.clone(),
        context: RefCell::new(Box::new(())),
        events: Rc::new(RefCell::new(Vec::new())),
        performance: performance
    });

    bind_event_handlers(
        &document,
        update_struct.events.clone(),
        update_struct.clone());

    update_clone(update_struct);
}

pub fn update(
    context : &mut Box<dyn Any>,
    input_events : &Vec<InputEvent>,
    rendering_context : &CanvasRenderingContext2d,
    time : f64) -> Expected<()> {
    let game_state = context.downcast_mut::<GameState>();

    if game_state.is_none() {
        *context = Box::new(GameState { 
            x: 0.0, y: 0.0, last_time: time,
            x_speed: 9.0, y_speed: 9.0 });
    }

    let game_state = context.downcast_mut::<GameState>()
        .ok_or(Error::Msg("Failed to downcast context to GameState!"))?;

        let canvas = rendering_context.canvas()
        .ok_or(Error::Msg("Failed to get canvas from rendering context."))?;

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    let canvas_size = vec2::vec2 { x : width as f32, y : height as f32 };

    game::update(game_state, input_events, canvas_size, time);
    game::render(game_state, rendering_context, canvas_size, time)?;

    return Ok(());
}