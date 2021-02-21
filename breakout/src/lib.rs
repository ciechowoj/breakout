extern crate nalgebra_glm as glm;
#[macro_use]
pub mod utils;
mod event;
mod game;
mod collision;
mod webapi;
mod executor;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::include_str;
use web_sys::*;
use utils::*;
use game::*;
use glm::vec2;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
        let style : HtmlStyleElement = style.unchecked_into();

        style.set_inner_html(&sheet);
    }
    else {
        game::utils::create_style_element(&document, &sheet, "font-css").unwrap();
    }
}

fn update_viewport_size() {
    let document = web_sys::window().unwrap().document().unwrap();
    let root = document.document_element().unwrap();

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

struct Application {
    js_performance : web_sys::Performance,
    last_update_time : f64,
    game_state : Option<std::rc::Rc<std::cell::RefCell<GameState>>>,
    update_closure : ClosureHandle
}

impl Application {
    fn new() -> std::rc::Rc<std::cell::RefCell<Self>> {
        let window = web_sys::window().unwrap();

        let application = std::rc::Rc::new(
            std::cell::RefCell::new(
                Application {
                    js_performance: window.performance().unwrap(),
                    last_update_time: 0f64,
                    game_state: None,
                    update_closure: ClosureHandle::Empty
                }));

        let closure = ClosureHandle::new({
            let window = window.clone();
            let application = std::rc::Rc::downgrade(&application);

            Box::new(move |_ : JsValue| {
                let application = application.upgrade();
                let application = application.unwrap();
                let mut application = application.borrow_mut();

                let time = now_sec(&application.js_performance);

                application.update(time).unwrap();

                let elapsed = now_sec(&application.js_performance) - time;

                crate::update_fps(
                    &mut application.last_update_time,
                    elapsed,
                    time).unwrap();

                window.request_animation_frame(application.update_closure.function())
                    .unwrap();
            })
        });

        application.borrow_mut().update_closure = closure;

        return application;
    }

    fn start(&mut self) {
        let window = window().unwrap();
        window.request_animation_frame(self.update_closure.function())
            .unwrap();
    }

    fn update(&mut self, time : f64) -> anyhow::Result<()> {
        let window = window().unwrap();
        let document = window.document().unwrap();

        let canvas : web_sys::HtmlCanvasElement = document.get_element_by_id("main-canvas-id")
            .unwrap().unchecked_into();

        let rendering_context : web_sys::CanvasRenderingContext2d = canvas.get_context("2d")
                .unwrap().unwrap().unchecked_into();

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

        if self.game_state.is_none() {
            self.game_state = Some(GameState::init(time));
            game::init_overlay(&mut self.game_state.as_mut().unwrap().borrow_mut(), time)?;
        }

        game::update(&mut self.game_state.as_mut().unwrap(), time)?;
        game::update_overlay(&mut self.game_state.as_mut().unwrap().borrow_mut(), time)?;
        game::render(&mut self.game_state.as_mut().unwrap().borrow_mut(), &rendering_context, canvas_size, time)?;

        return Ok(());
    }
}

#[wasm_bindgen]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    set_panic_hook();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    document.set_title("Rusty Breakout");

    game::utils::create_style_element(&document, include_str!("main.css"), "main-css")
        .expect("Failed to create main.css.");

    let body = document.body().unwrap();

    let outer_div : HtmlElement = document.create_element("div")
        .unwrap().unchecked_into();

    outer_div.set_id("outer-div");
    body.append_child(&outer_div).ok();

    let canvas = document.create_element("canvas").unwrap();
    let canvas : web_sys::HtmlCanvasElement = canvas.unchecked_into();
    canvas.set_id("main-canvas-id");
    canvas.set_class_name("main-canvas-area");
    reset_canvas_size(&canvas).unwrap();

    outer_div.append_child(&canvas).ok();

    let overlay : HtmlElement = document.create_element("div").unwrap().unchecked_into();
    overlay.set_id("main-overlay-id");
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

    let application = Application::new();

    application.borrow_mut().start();
    std::mem::forget(application);
}

pub fn update_fps(
    last_time : &mut f64,
    elapsed : f64,
    time : f64) -> anyhow::Result<()> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let overlay : HtmlElement = document.get_element_by_id("main-overlay-id")
        .unwrap().unchecked_into();

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
