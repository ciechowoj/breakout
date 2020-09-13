extern crate nalgebra_glm as glm;
mod event;
#[macro_use]
mod utils;
mod dom_utils;
mod game;
mod collision;
mod webapi;
mod executor;

use std::any::Any;
use std::cell::RefCell;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::{Rc};
use std::include_str;
use web_sys::*;
use event::*;
use utils::*;
use game::*;
use crate::dom_utils::*;
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

fn reset_canvas_size(canvas : &HtmlCanvasElement) -> anyhow::Result<()> {
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
pub async fn wasm_main() {
    console_error_panic_hook::set_once();

    struct Recursive {
        value: Rc<dyn Fn(Rc<Recursive>)>,
        context: RefCell<Box<dyn Any>>,
        events: Rc<RefCell<Vec<InputEvent>>>,
        event_queues: Rc<RefCell<EventQueues>>,
        performance: Performance,
        overlay: HtmlElement,
        last_time: RefCell<f64>
    };

    fn bind_event_handlers(
        document : &Document,
        canvas : &HtmlCanvasElement,
        overlay : &HtmlElement,
        input_events : Rc<RefCell<Vec<InputEvent>>>,
        update_struct : Rc<Recursive>) {

        fn get_code(js_value : JsValue) -> anyhow::Result<KeyCode> {
            let code = js_sys::Reflect::get(&js_value, &JsValue::from_str("code")).to_anyhow()?;
            let code = code.as_string().ok_or(anyhow::anyhow!("Expected 'code' field in KeyboardEvent!"))?;
            let code = KeyCode::from_str(&code.to_string())?;
            return Ok(code);
        };

        fn js_to_keydown_event(js_value : JsValue) -> anyhow::Result<InputEvent> {
            let event = InputEvent::KeyDown { 
                code: get_code(js_value)?
            }; 

            return Ok(event);
        }

        fn js_to_keyup_event(js_value : JsValue) -> anyhow::Result<InputEvent> {
            let event = InputEvent::KeyUp {
                code: get_code(js_value)?
            };

            return Ok(event);
        }

        fn js_to_mousemove_event(js_value : JsValue) -> anyhow::Result<InputEvent> {
            let offset_x = js_sys::Reflect::get(&js_value, &JsValue::from_str("offsetX")).to_anyhow()?;
            let offset_x = offset_x.as_f64().ok_or(anyhow::anyhow!("Expected 'offsetX' field in MouseEvent!"))?;

            let offset_y = js_sys::Reflect::get(&js_value, &JsValue::from_str("offsetY")).to_anyhow()?;
            let offset_y = offset_y.as_f64().ok_or(anyhow::anyhow!("Expected 'offsetY' field in MouseEvent!"))?;

            let event = InputEvent::MouseMove { 
                x: offset_x,
                y: offset_y
            };

            return Ok(event);
        }
    
        let on_keydown : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_keydown_event(js_value).unwrap();
            input_events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_keydown as Box<dyn FnMut(JsValue)>);
        document.set_onkeydown(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let update_struct_clone = update_struct.clone();
        let on_keyup : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_keyup_event(js_value).unwrap();
            update_struct_clone.events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_keyup as Box<dyn FnMut(JsValue)>);
        document.set_onkeyup(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let update_struct_clone = update_struct.clone();
        let on_mousemove : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {            
            let event = js_to_mousemove_event(js_value).unwrap();
            update_struct_clone.events.borrow_mut().push(event);
        });
        
        let closure = Closure::wrap(on_mousemove as Box<dyn FnMut(JsValue)>);
        canvas.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
        closure.forget();

        let on_touchstart : Box<dyn FnMut(event::TouchEvent) -> anyhow::Result<()>> = Box::new(move |value : event::TouchEvent| -> anyhow::Result<()> {
            for touch in value.touches {
                log!("on_touchstart: {:?}", touch);
            }

            return Ok(());
        });

        let set_ontouchmove : Box<dyn FnMut(event::TouchEvent) -> anyhow::Result<()>> = Box::new(move |value : event::TouchEvent| -> anyhow::Result<()> {
            for touch in value.touches {
                log!("set_ontouchmove: {:?}", touch);
            }

            return Ok(());
        });

        let set_ontouchend : Box<dyn FnMut(event::TouchEvent) -> anyhow::Result<()>> = Box::new(move |value : event::TouchEvent| -> anyhow::Result<()> {
            for touch in value.touches {
                log!("set_ontouchend: {:?}", touch);
            }

            return Ok(());
        });

        let set_ontouchcancel : Box<dyn FnMut(event::TouchEvent) -> anyhow::Result<()>> = Box::new(move |value : event::TouchEvent| -> anyhow::Result<()> {
            for touch in value.touches {
                log!("set_ontouchcancel: {:?}", touch);
            }

            return Ok(());
        });

        <HtmlElement as InputEventTarget>::set_ontouchstart(overlay.as_ref(), on_touchstart);
        <HtmlElement as InputEventTarget>::set_ontouchmove(overlay.as_ref(), set_ontouchmove);
        <HtmlElement as InputEventTarget>::set_ontouchend(overlay.as_ref(), set_ontouchend);
        <HtmlElement as InputEventTarget>::set_ontouchcancel(overlay.as_ref(), set_ontouchcancel);
    }

    fn setup_main_loop(
        document : &Document,
        canvas : &HtmlCanvasElement,
        overlay : HtmlElement,
        window : Window) -> anyhow::Result<()> {
    
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
                        &update.event_queues.borrow(),
                        &rendering_context,
                        &update.overlay,
                        time).unwrap();

                    EventQueues::clear_all_queues(&update.event_queues);
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
            event_queues: EventQueues::new(),
            performance: performance,
            overlay: overlay.clone(),
            last_time: RefCell::new(0f64)
        });
    
        bind_event_handlers(
            &document,
            &canvas,
            &overlay,
            update_struct.events.clone(),
            update_struct.clone());
    
        EventQueues::bind_all_queues(Rc::downgrade(&update_struct.event_queues), &overlay);

        update_clone(update_struct);

        return Ok(());
    }

    set_panic_hook();

    let window = web_sys::window().expect("no global `window` exists");
    
    let document = window.document().expect("should have a document on window");
    document.set_title("Omg! It works!");

    create_style_element(&document, include_str!("main.css"), "main-css")
        .expect("Failed to create main.css.");

    let body = document.body().expect("document should have a body");
    
    let outer_div = document.create_element("div").unwrap();
    let outer_div = outer_div.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();

    outer_div.set_id("outer-div");
    body.append_child(&outer_div).ok();

    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_class_name("main-canvas-area");
    reset_canvas_size(&canvas).unwrap();
    
    outer_div.append_child(&canvas).ok();
    
    let overlay = document.create_element("div").unwrap();
    let overlay = overlay.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();
    overlay.set_class_name("main-canvas-area");
    outer_div.append_child(&overlay).ok();
        
    setup_main_loop(&document, &canvas, overlay, window).unwrap();
}

pub fn update(
    context : &mut Box<dyn Any>,
    input_events : &Vec<InputEvent>,
    event_queues : &EventQueues,
    rendering_context : &CanvasRenderingContext2d,
    overlay : &HtmlElement,
    time : f64) -> anyhow::Result<()> {
    
    let canvas = rendering_context.canvas()
        .ok_or(anyhow::anyhow!("Failed to get canvas from rendering context."))?;

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
            .ok_or(anyhow::anyhow!("Failed to downcast context to GameState!"))?;
        game::init_overlay(game_state, overlay, time)?;
    }

    let game_state = context.downcast_mut::<GameState>()
        .ok_or(anyhow::anyhow!("Failed to downcast context to GameState!"))?;

    game::update(game_state, input_events, event_queues, canvas_size, time)?;
    game::update_overlay(game_state, overlay, time)?;
    game::render(game_state, rendering_context, canvas_size, time)?;

    return Ok(());
}

pub fn update_fps(
    last_time : &mut f64,
    overlay : &HtmlElement,
    elapsed : f64,
    time : f64) -> anyhow::Result<()> {

    let document = overlay.owner_document().ok_or(anyhow::anyhow!("Failed to get document node."))?;

    if let None = document.get_element_by_id("fps-counter") {
        let fps_counter = document.create_element("span").unwrap();
        let fps_counter = fps_counter.dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| ())
            .unwrap();

        fps_counter.set_id("fps-counter");
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