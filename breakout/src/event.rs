use crate::utils::*;

use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom};
use std::rc::{Rc, Weak};

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

#[derive(Debug)]
pub enum TouchEventType {
    TouchCancel,
    TouchEnd,
    TouchMove,
    TouchStart
}

#[derive(Debug)]
pub struct TouchEvent {
    pub r#type : TouchEventType,
    pub touches : Vec<Touch>
}

impl TryFrom<String> for TouchEventType {
    type Error = anyhow::Error;

    fn try_from(r#type: String) -> anyhow::Result<Self> {
        let error = "Failed to convert JsValue to TouchEventType!";

        match r#type.as_str() {
            "touchcancel" => Ok(TouchEventType::TouchCancel),
            "touchend" => Ok(TouchEventType::TouchEnd),
            "touchmove" => Ok(TouchEventType::TouchMove),
            "touchstart" => Ok(TouchEventType::TouchStart),
            _ => Err(anyhow::anyhow!(error))
        }
    }
}

impl TryFrom<web_sys::TouchEvent> for TouchEvent {
    type Error = anyhow::Error;

    fn try_from(touch_event: web_sys::TouchEvent) -> anyhow::Result<Self> {

        let touches = touch_event.touches();

        let mut output_touches : Vec<Touch> = Vec::with_capacity(touches.length() as usize);

        for i in 0..touches.length() {
            output_touches.push(Touch::try_from(touches.get(i).unwrap())?);
        }

        Ok(TouchEvent {
            r#type: TouchEventType::try_from(touch_event.type_())?,
            touches: output_touches
        })
    }
}

pub struct EventQueues {
    pub touch_events : Vec<TouchEvent>
}

impl EventQueues {
    pub fn new() -> Rc<RefCell<EventQueues>> {
        let event_queues = EventQueues {
            touch_events: Vec::<TouchEvent>::new()
        };

        return Rc::new(RefCell::new(event_queues))
    }

    pub fn bind_all_queues(event_queues : Weak<RefCell<EventQueues>>, source : &HtmlElement) {
        let closure = ClosureHandle::new(Box::new(move |event : web_sys::TouchEvent| {
            match event_queues.upgrade() {
                Some(event_queues) => event_queues.borrow_mut().touch_events.push(TouchEvent::try_from(event).unwrap()),
                None => ()
            }
        }));

        source.add_event_listener_with_callback("touchstart", closure.function()).unwrap();
        source.add_event_listener_with_callback("touchmove", closure.function()).unwrap();
        source.add_event_listener_with_callback("touchend", closure.function()).unwrap();
        source.add_event_listener_with_callback("touchcancel", closure.function()).unwrap();

        std::mem::forget(closure);
    }

    pub fn clear_all_queues(event_queues : &Rc<RefCell<EventQueues>>) {
        let mut event_queues = event_queues.borrow_mut();
        event_queues.touch_events.clear();
    }
}

pub struct TouchTracker {
    pub touches : Vec<Touch>
}

impl TouchTracker {
    pub fn new() -> TouchTracker {
        TouchTracker { touches: Vec::new() }
    }

    pub fn update(&mut self, touch_events : &Vec<TouchEvent>) {
        let mut id_map : HashMap<i32, usize> = HashMap::new();

        for i in 0..self.touches.len() {
            id_map.insert(self.touches[i].identifier, i);
        }

        for event in touch_events {
            match event.r#type {
                TouchEventType::TouchStart | TouchEventType::TouchMove  => {
                    for touch in &event.touches {
                        if let Some(index) = id_map.get(&touch.identifier) {
                            self.touches[*index] = *touch;
                        }
                        else {
                            id_map.insert(touch.identifier, self.touches.len());
                            self.touches.push(*touch);
                        }
                    }
                }

                TouchEventType::TouchEnd | TouchEventType::TouchCancel => {
                    let mut retain_set : HashSet<i32> = HashSet::new();

                    for touch in &event.touches {
                        retain_set.insert(touch.identifier);
                    }

                    let mut i = 0;
                    while i < self.touches.len() {
                        if !retain_set.contains(&self.touches[i].identifier) {
                            id_map.remove(&self.touches[i].identifier);
                            self.touches[i] = self.touches[self.touches.len() - 1];
                            id_map.insert(self.touches[i].identifier, i);
                            self.touches.pop();
                        }
                        else {
                            i += 1;
                        }
                    }
                }
            }
        }
    }
}

pub struct KeyboardState {
    state : HashSet<String>
}

impl KeyboardState {
    pub fn new() -> Rc<RefCell<KeyboardState>> {
        let keyboard_state = Rc::new(RefCell::new(KeyboardState { state: HashSet::new() }));
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        {
            let keyboard_state = keyboard_state.clone();

            let on_keydown : Box<dyn FnMut(web_sys::KeyboardEvent)> = Box::new(move |event : web_sys::KeyboardEvent| {
                keyboard_state.borrow_mut().state.insert(event.key());
            });

            let closure = Closure::wrap(on_keydown);
            document.add_event_listener_with_callback("keydown", closure.as_ref()
                .unchecked_ref()).unwrap();

            closure.forget();
        }

        {
            let keyboard_state = keyboard_state.clone();

            let on_keyup : Box<dyn FnMut(web_sys::KeyboardEvent)> = Box::new(move |event : web_sys::KeyboardEvent| {
                keyboard_state.borrow_mut().state.remove(&event.key());
            });

            let closure = Closure::wrap(on_keyup);
            document.add_event_listener_with_callback("keyup", closure.as_ref()
                .unchecked_ref()).unwrap();
            closure.forget();
        }

        return keyboard_state;
    }

    pub fn is_down(&self, code : &'static str) -> bool {
       self.state.contains(code)
    }
}
