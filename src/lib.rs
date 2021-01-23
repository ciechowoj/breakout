extern crate nalgebra_glm as glm;
mod event;
#[macro_use]
pub mod utils;
mod js_utils;
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
use crate::js_utils::*;
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

fn update_dynamic_fonts(width : i32) {
    let document = web_sys::window().unwrap().document().unwrap();

    let scale = ((if width < 480 { 480 } else { width }) as f32) * 0.012f32;
    let vhuge_font_size = (8.71855f32 * scale) as i32;
    let huge_font_size = (6.85410f32 * scale) as i32;
    let vlarge_font_size = (5.38836f32 * scale) as i32;
    let large_font_size = (4.23606f32 * scale) as i32;
    let normal_font_size = (3.33019f32 * scale) as i32;
    let small_font_size = (2.61803f32 * scale) as i32;
    let footnote_font_size = (2.05817f32 * scale) as i32;
    let script_font_size = (1.27201f32 * scale) as i32;
    let tiny_font_size = (1f32 * scale) as i32;

    let sheet = format!(r#"
:root {{
    --font-vhuge: {}px;
    --font-huge: {}px;
    --font-vlarge: {}px;
    --font-large: {}px;
    --font-normal: {}px;
    --font-small: {}px;
    --font-footnote: {}px;
    --font-script: {}px;
    --font-tiny: {}px;
}}"#,   vhuge_font_size,
        huge_font_size,
        vlarge_font_size,
        large_font_size,
        normal_font_size,
        small_font_size,
        footnote_font_size,
        script_font_size,
        tiny_font_size);

    let fonts_css = document.get_element_by_id("font-css");

    if let Some(style) = fonts_css {
        let style : HtmlStyleElement = style.xdyn_into();

        style.set_inner_html(&sheet);
    }
    else {
        game::utils::create_style_element(&document, &sheet, "font-css").unwrap();
    }
}

fn update_viewport_size() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document
        .document_element()
        .expect("Failed to retrieve reference to the html element.");

    let outer_div : HtmlElement = document.get_element_by_id("outer-div").unwrap().unchecked_into();

    let client_width = root.client_width() as f32;
    let client_height = root.client_height() as f32;

    let aspect = 3f32 / 4f32; // w / h

    let width;
    let height;

    if client_width < client_height * aspect {
        width = client_width as i32;
        height = (client_width / aspect) as i32;
    }
    else {
        width = (client_height * aspect) as i32;
        height = client_height as i32;
    }

    update_dynamic_fonts(width);

    let width = format!("{}px", width);
    let height = format!("{}px", height);

    outer_div.style().set_property("width", width.as_ref()).to_anyhow().unwrap();
    outer_div.style().set_property("height", height.as_ref()).to_anyhow().unwrap();
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
            let rendering_context : web_sys::CanvasRenderingContext2d = canvas_clone
                .get_context("2d")
                .unwrap()
                .unwrap()
                .xdyn_into();

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

    let window = web_sys::window().unwrap();

    let document = window.document().expect("should have a document on window");
    document.set_title("Omg! It works!");

    game::utils::create_style_element(&document, include_str!("main.css"), "main-css")
        .expect("Failed to create main.css.");

    let body = document.body().expect("document should have a body");

    let outer_div : HtmlElement = document.create_element("div").unwrap().unchecked_into();

    outer_div.set_id("outer-div");
    body.append_child(&outer_div).ok();

    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Failed to cast 'Element' to 'HtmlCanvasElement'.");

    canvas.set_class_name("main-canvas-area");
    reset_canvas_size(&canvas).unwrap();

    outer_div.append_child(&canvas).ok();

    let overlay : HtmlElement = document.create_element("div").unwrap().unchecked_into();
    overlay.set_class_name("main-canvas-area");
    outer_div.append_child(&overlay).ok();

    let event_target = EventTarget::from(web_sys::window().unwrap());

    let closure : Box<dyn Fn(JsValue)> = Box::new(|_event : JsValue| {
        update_viewport_size();
    });

    closure(JsValue::NULL);

    let closure = Closure::wrap(closure);
    let function = closure.as_ref().unchecked_ref();
    event_target.add_event_listener_with_callback("resize", function).to_anyhow().unwrap();
    closure.forget();

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
        *context = Box::new(GameState::init(time));
        let game_state = context.downcast_mut::<GameState>()
            .ok_or(anyhow::anyhow!("Failed to downcast context to GameState!"))?;
        game::init_overlay(game_state, overlay, time)?;
    }

    let game_state = context.downcast_mut::<GameState>()
        .ok_or(anyhow::anyhow!("Failed to downcast context to GameState!"))?;

    game::update(game_state, overlay, input_events, event_queues, time)?;
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
        let fps_counter : HtmlElement = document.create_element("span").unwrap().unchecked_into();

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