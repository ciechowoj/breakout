extern crate nalgebra_glm as glm;
#[macro_use]
mod utils;
mod game;
mod collision;

use std::any::Any;
use std::cell::RefCell;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use web_sys::*;
use utils::*;
use game::*;
use glm::vec2;

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

fn reset_canvas_size(canvas : &HtmlCanvasElement) -> Expected<()> {
    let mut scroll_width = canvas.scroll_width();
    let mut scroll_height = canvas.scroll_height();

    if scroll_width <= 0 {
        scroll_width = 640;
    }

    if scroll_height <= 0 {
        scroll_height = 480;
    }

    canvas.set_width(scroll_width as u32);
    canvas.set_height(scroll_height as u32);

    return Ok(());
}

#[wasm_bindgen]
pub fn greet() {
    fn set_body_style(body : &HtmlElement) -> Expected<()> {
        body.style().set_property("margin", "0px")?;
        body.style().set_property("padding", "0px")?;
        body.style().set_property("width", "100%")?;
        body.style().set_property("height", "1210px")?;
        return Ok(());
    }

    fn set_canvas_style(canvas : &HtmlElement) -> Expected<()> {
        canvas.style().set_property("border", "none")?;
        canvas.style().set_property("width", "100%")?;
        canvas.style().set_property("height", "100%")?;
        canvas.style().set_property("margin-left", "0px")?;
        canvas.style().set_property("margin-right", "0px")?;
        canvas.style().set_property("padding-left", "0px")?;
        canvas.style().set_property("padding-right", "0px")?;
        canvas.style().set_property("display", "block")?;
        canvas.style().set_property("position", "absolute")?;
        return Ok(());
    }

    fn set_outer_div_style(canvas : &HtmlElement) -> Expected<()> {
        canvas.style().set_property("border", "none")?;
        canvas.style().set_property("min-width", "1000px")?;
        canvas.style().set_property("min-height", "1210px")?;
        canvas.style().set_property("width", "1000px")?;
        canvas.style().set_property("height", "1210px")?;
        canvas.style().set_property("margin-left", "auto")?;
        canvas.style().set_property("margin-right", "auto")?;
        canvas.style().set_property("padding-left", "0px")?;
        canvas.style().set_property("padding-right", "0px")?;
        canvas.style().set_property("display", "block")?;
        canvas.style().set_property("position", "relative")?;
        return Ok(());
    }

    struct Recursive {
        value: Rc<dyn Fn(Rc<Recursive>)>,
        context: RefCell<Box<dyn Any>>,
        events: Rc<RefCell<Vec<InputEvent>>>,
        performance: Performance,
        overlay: HtmlElement,
        last_time: RefCell<f64>
    };

    fn bind_event_handlers(
        document : &Document,
        canvas : &HtmlCanvasElement,
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

        fn js_to_mousemove_event(js_value : JsValue, performance : &Performance) -> Expected<InputEvent> {
            let offset_x = js_sys::Reflect::get(&js_value, &JsValue::from_str("offsetX"))?;
            let offset_x = offset_x.as_f64().ok_or(Error::Msg("Expected 'offsetX' field in MouseEvent!"))?;

            let offset_y = js_sys::Reflect::get(&js_value, &JsValue::from_str("offsetY"))?;
            let offset_y = offset_y.as_f64().ok_or(Error::Msg("Expected 'offsetY' field in MouseEvent!"))?;

            let event = InputEvent::MouseMove { 
                time: now_sec(performance),
                x: offset_x,
                y: offset_y
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

        let update_struct_clone = update_struct.clone();
        let on_mousemove : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_mousemove_event(
                js_value,
                &update_struct_clone.performance).unwrap();
    
            update_struct_clone.events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_mousemove as Box<dyn FnMut(JsValue)>);
        canvas.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    fn setup_main_loop(
        document : &Document,
        canvas : &HtmlCanvasElement,
        overlay : HtmlElement,
        window : Window) -> Expected<()> {
    
        let performance = window.performance().expect("performance should be available");

        let canvas_clone = canvas.clone();
    
        let update = move |update: Rc<Recursive>| {
            let rendering_context = canvas_clone
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
                        &update.overlay,
                        time).unwrap();

                    update.events.borrow_mut().clear();

                    let elapsed = now_sec(&update.performance) - time;
                    
                    crate::update_fps(
                        &mut update.last_time.borrow_mut(),
                        &update.overlay,
                        elapsed,
                        time).unwrap();

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
            performance: performance,
            overlay: overlay,
            last_time: RefCell::new(0f64)
        });
    
        bind_event_handlers(
            &document,
            &canvas,
            update_struct.events.clone(),
            update_struct.clone());
    
        update_clone(update_struct);

        return Ok(());
    }

    set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    
    let document = window.document().expect("should have a document on window");
    document.set_title("Omg! It works!");

    let html = document.document_element().expect("document should have a html");
    let html = html.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();

    html.style().set_property("height", "100%").unwrap();

    let body = document.body().expect("document should have a body");
    set_body_style(&body).unwrap();
    
    let outer_div = document.create_element("div").unwrap();
    let outer_div = outer_div.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();
    set_outer_div_style(&outer_div).unwrap();
    body.append_child(&outer_div).ok();

    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    reset_canvas_size(&canvas).unwrap();
    set_canvas_style(&canvas).unwrap();
    
    outer_div.append_child(&canvas).ok();
    
    let overlay = document.create_element("div").unwrap();
    let overlay = overlay.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();
    set_canvas_style(&overlay).unwrap();
    outer_div.append_child(&overlay).ok();
        
    setup_main_loop(&document, &canvas, overlay, window).unwrap();
}

pub fn update(
    context : &mut Box<dyn Any>,
    input_events : &Vec<InputEvent>,
    rendering_context : &CanvasRenderingContext2d,
    overlay : &HtmlElement,
    time : f64) -> Expected<()> {
    
    let canvas = rendering_context.canvas()
        .ok_or(Error::Msg("Failed to get canvas from rendering context."))?;

    let mut width = canvas.width();
    let mut height = canvas.height();
    let scroll_width = canvas.scroll_width() as u32;
    let scroll_height = canvas.scroll_height() as u32;

    if width != scroll_width || height != scroll_height {
        reset_canvas_size(&canvas)?;
        width = canvas.width();
        height = canvas.height();
    }

    let canvas_size = vec2(width as f32, height as f32);

    let game_state = context.downcast_mut::<GameState>();

    if game_state.is_none() {
        *context = Box::new(init(canvas_size, time));
        let game_state = context.downcast_mut::<GameState>()
            .ok_or(Error::Msg("Failed to downcast context to GameState!"))?;
        game::init_overlay(game_state, overlay, time)?;
    }

    let game_state = context.downcast_mut::<GameState>()
        .ok_or(Error::Msg("Failed to downcast context to GameState!"))?;

    game::update(game_state, input_events, canvas_size, time)?;
    game::update_overlay(game_state, overlay, time)?;
    game::render(game_state, rendering_context, canvas_size, time)?;

    return Ok(());
}

pub fn update_fps(
    last_time : &mut f64,
    overlay : &HtmlElement,
    elapsed : f64,
    time : f64) -> Expected<()> {

    let document = overlay.owner_document().ok_or(Error::Msg("Failed to get document node."))?;

    if let None = document.get_element_by_id("fps-counter") {
        let fps_counter = document.create_element("span").unwrap();
        let fps_counter = fps_counter.dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| ())
            .unwrap();

        fps_counter.set_id("fps-counter");
        fps_counter.style().set_property("font-family", "\"Lucida Console\", Courier, monospace")?;
        fps_counter.style().set_property("font-size", "16px")?;
        overlay.append_child(&fps_counter).ok();       
    }

    if let Some(fps_counter) = document.get_element_by_id("fps-counter") {
        let frame_time = time - *last_time;

        let frame_time = format!(
            "FPS: {:.4}</br>Frame time: {:.4}</br>Game time: {:.4} [{:.0}%]",
            1f64 / frame_time,
            frame_time,
            elapsed,
            elapsed / frame_time * 100f64);

        fps_counter.set_inner_html(&frame_time[..]);

    }

    *last_time = time;

    return Ok(());
}