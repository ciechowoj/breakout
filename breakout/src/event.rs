use crate::utils::*;

use web_sys::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom};
use std::rc::{Rc};

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    pub identifier : i32,
    pub screen_x : i32,
    pub screen_y : i32,
    pub client_x : i32,
    pub client_y : i32,
    pub page_x : i32,
    pub page_y : i32
}

impl TryFrom<web_sys::Touch> for Touch {
    type Error = anyhow::Error;

    fn try_from(touch: web_sys::Touch) -> anyhow::Result<Self> {
        Ok(Touch {
            identifier: touch.identifier(),
            screen_x: touch.screen_x(),
            screen_y: touch.screen_y(),
            client_x: touch.client_x(),
            client_y: touch.client_y(),
            page_x: touch.page_x(),
            page_y: touch.page_y()
        })
    }
}

pub struct TouchTracker {
    pub touches : Vec<Touch>,
    pub closure : ClosureHandle
}

impl TouchTracker {
    pub fn new() -> std::rc::Rc<std::cell::RefCell<TouchTracker>> {
        let touch_tracker = Rc::new(RefCell::new(
            TouchTracker {
                touches: Vec::new(),
                closure: ClosureHandle::Empty
            }));

        let closure = ClosureHandle::new({
            let touch_tracker = std::rc::Rc::downgrade(&touch_tracker);

            Box::new(move |event : web_sys::TouchEvent| {
                let mut id_map : HashMap<i32, usize> = HashMap::new();
                let touch_tracker = touch_tracker.upgrade().unwrap();
                let mut touch_tracker = touch_tracker.borrow_mut();
                let event_touches = event.touches();

                let touches = (0..event_touches.length()).map(|i| {
                    Touch::try_from(event_touches.get(i).unwrap()).unwrap()
                });

                match event.type_().as_str() {
                    "touchstart" | "touchmove" => {
                        for touch in touches {
                            if let Some(index) = id_map.get(&touch.identifier) {
                                touch_tracker.touches[*index] = touch;
                            }
                            else {
                                id_map.insert(touch.identifier, touch_tracker.touches.len());
                                touch_tracker.touches.push(touch);
                            }
                        }
                    },

                    "touchend" | "touchcancel" => {
                        let mut retain_set : HashSet<i32> = HashSet::new();

                        for touch in touches {
                            retain_set.insert(touch.identifier);
                        }

                        let mut i = 0;
                        while i < touch_tracker.touches.len() {
                            if !retain_set.contains(&touch_tracker.touches[i].identifier) {
                                id_map.remove(&touch_tracker.touches[i].identifier);
                                touch_tracker.touches[i] = touch_tracker.touches[touch_tracker.touches.len() - 1];
                                id_map.insert(touch_tracker.touches[i].identifier, i);
                                touch_tracker.touches.pop();
                            }
                            else {
                                i += 1;
                            }
                        }
                    }

                    _ => ()
                }
            })
        });

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let overlay : HtmlElement = document.get_element_by_id("main-overlay-id")
            .unwrap().unchecked_into();

        overlay.add_event_listener_with_callback("touchstart", closure.function()).unwrap();
        overlay.add_event_listener_with_callback("touchmove", closure.function()).unwrap();
        overlay.add_event_listener_with_callback("touchend", closure.function()).unwrap();
        overlay.add_event_listener_with_callback("touchcancel", closure.function()).unwrap();

        let body : HtmlElement = document.body().unwrap();

        body.add_event_listener_with_callback("touchstart", closure.function()).unwrap();
        body.add_event_listener_with_callback("touchmove", closure.function()).unwrap();
        body.add_event_listener_with_callback("touchend", closure.function()).unwrap();
        body.add_event_listener_with_callback("touchcancel", closure.function()).unwrap();

        touch_tracker.borrow_mut().closure = closure;

        return touch_tracker;
    }
}

pub struct KeyboardState {
    state : HashSet<String>,
    keydown_closure : ClosureHandle,
    keyup_closure : ClosureHandle
}

impl KeyboardState {
    pub fn new() -> Rc<RefCell<KeyboardState>> {
        let keyboard_state = Rc::new(RefCell::new(
            KeyboardState {
                state: HashSet::new(),
                keydown_closure: ClosureHandle::Empty,
                keyup_closure: ClosureHandle::Empty
            }));

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let keydown_closure = ClosureHandle::new({
            let keyboard_state = std::rc::Rc::downgrade(&keyboard_state);

            Box::new(move |event : web_sys::KeyboardEvent| {
                let keyboard_state = keyboard_state.upgrade().unwrap();
                keyboard_state.borrow_mut().state.insert(event.key());
            })
        });

        let keyup_closure = ClosureHandle::new({
            let keyboard_state = std::rc::Rc::downgrade(&keyboard_state);

            Box::new(move |event : web_sys::KeyboardEvent| {
                let keyboard_state = keyboard_state.upgrade().unwrap();
                keyboard_state.borrow_mut().state.remove(&event.key());
            })
        });

        document.add_event_listener_with_callback("keydown", keydown_closure.function()).unwrap();
        document.add_event_listener_with_callback("keyup", keyup_closure.function()).unwrap();

        keyboard_state.borrow_mut().keydown_closure = keydown_closure;
        keyboard_state.borrow_mut().keyup_closure = keyup_closure;

        return keyboard_state;
    }

    pub fn is_down(&self, code : &'static str) -> bool {
       self.state.contains(code)
    }
}
