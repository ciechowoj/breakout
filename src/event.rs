use js_sys::{Array, Function};
use crate::utils::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use strum_macros::AsRefStr;
use strum_macros::EnumString;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom};
use std::rc::{Rc, Weak};

#[derive(Clone, Copy, Debug, AsRefStr, EnumString, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Again,
    AltLeft,
    AltRight,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    Backquote,
    Backslash,
    Backspace,
    BracketLeft,
    BracketRight,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    CapsLock,
    Comma,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Convert,
    Copy,
    Cut,
    Delete,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Eject,
    End,
    Enter,
    Equal,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F2,
    F20,
    F21,
    F22,
    F23,
    F24,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Find,
    Help,
    Home,
    Insert,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KanaMode,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Lang1,
    Lang2,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    LaunchMediaPlayer,
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Minus,
    NonConvert,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadChangeSign,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadSubtract,
    Open,
    OSLeft,
    OSRight,
    PageDown,
    PageUp,
    Paste,
    Pause,
    Period,
    Power,
    PrintScreen,
    Props,
    Quote,
    ScrollLock,
    Select,
    Semicolon,
    ShiftLeft,
    ShiftRight,
    Slash,
    Space,
    Tab,
    Undo,
    WakeUp
}

#[derive(Debug, Clone, Copy)]
pub struct Touch {
    pub identifier : u32,
    pub screen_x : f32,
    pub screen_y : f32,
    pub client_x : f32,
    pub client_y : f32,
    pub page_x : f32,
    pub page_y : f32
}

impl TryFrom<JsValue> for Touch {
    type Error = Error;

    fn try_from(js_value: JsValue) -> Expected<Self> {
        Ok(Touch {
            identifier: get_property_as_f64(&js_value, "identifier")? as u32,
            screen_x: get_property_as_f64(&js_value, "screenX")? as f32,
            screen_y: get_property_as_f64(&js_value, "screenY")? as f32,
            client_x: get_property_as_f64(&js_value, "clientX")? as f32,
            client_y: get_property_as_f64(&js_value, "clientY")? as f32,
            page_x: get_property_as_f64(&js_value, "pageX")? as f32,
            page_y: get_property_as_f64(&js_value, "pageY")? as f32
        })
    }
}

pub enum InputEvent {
    KeyDown { code : KeyCode },
    KeyUp { code : KeyCode },
    MouseMove { x : f64, y : f64 },
    
}

#[derive(Debug)]
pub enum KeyboardEventType {
    KeyDown,
    KeyPress,
    KeyUp
}

#[derive(Debug)]
pub struct KeyboardEvent {
    pub r#type : KeyboardEventType,
    pub code : KeyCode
}

#[derive(Debug)]
pub enum MouseEventType {

}

#[derive(Debug)]
pub struct MouseEvent {
    pub r#type : MouseEventType,
    pub code : KeyCode
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

impl TryFrom<JsValue> for TouchEventType {
    type Error = Error;

    fn try_from(js_value: JsValue) -> Expected<Self> {
        let error = "Failed to convert JsValue to TouchEventType!";
        let r#type = js_value.as_string()
            .ok_or(Error::Msg(error))?;

        match r#type.as_str() {
            "touchcancel" => Ok(TouchEventType::TouchCancel),
            "touchend" => Ok(TouchEventType::TouchEnd),
            "touchmove" => Ok(TouchEventType::TouchMove),
            "touchstart" => Ok(TouchEventType::TouchStart),
            _ => Err(Error::Msg(error))
        }
    }
}

impl TryFrom<JsValue> for TouchEvent {
    type Error = Error;

    fn try_from(js_value: JsValue) -> Expected<Self> {
        let touches_property = get_property(&js_value, "touches")?;
        let item_js_value = get_property(&touches_property, "item")?;
        let item_method : &Function = item_js_value.as_ref().unchecked_ref();
        let length_property = get_property_as_usize(&touches_property, "length")?;

        let array = Array::new_with_length(1);
        let mut touches : Vec<Touch> = Vec::with_capacity(length_property);

        for i in 0..length_property {
            array.set(0, JsValue::from_f64(i as f64));

            let js_result = js_sys::Reflect::apply(
                &item_method,
                &touches_property,
                &array)?;

            touches.push(Touch::try_from(js_result)?);
        }

        Ok(TouchEvent { 
            r#type: TouchEventType::try_from(get_property(&js_value, "type")?)?,
            touches: touches 
        })
    }
}

pub trait InputEventTarget {
    fn set_onmousemove(&self, function : Box<dyn FnMut(InputEvent) -> ExpectedUnit>);
    fn set_ontouchstart(&self, function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>);
    fn set_ontouchmove(&self, function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>);
    fn set_ontouchend(&self, function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>);
    fn set_ontouchcancel(&self, function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>);
}

fn get_property(js_value : &JsValue, id : &'static str) -> Expected<JsValue> {
    let property = js_sys::Reflect::get(&js_value, &JsValue::from_str(id))?;
    return Ok(property);
}

fn get_property_as_f64(js_value : &JsValue, id : &'static str) -> Expected<f64> {
    fn error_message(id : &'static str) -> Error {
        let mut message = "Cannot convert the '".to_owned();
        message.push_str(id);
        message.push_str("' property to f64!");
        return Error::Str(message);
    }

    let property = get_property(js_value, id)?;

    return match property.as_f64() {
        Some(value) => Ok(value),
        None => Err(error_message(id))
    };
}

fn get_property_as_usize(js_value : &JsValue, id : &'static str) -> Expected<usize> {
    fn error_message(id : &'static str) -> Error {
        let mut message = "Cannot convert the '".to_owned();
        message.push_str(id);
        message.push_str("' property to usize!");
        return Error::Str(message);
    }

    let property = get_property(js_value, id)?;

    return match property.as_f64() {
        Some(value) => {
            match usize::min_value() as f64 <= value && value <= usize::max_value() as f64 && value.fract() == 0f64 {
                true => Ok(value as usize),
                false => Err(error_message(id))
            }
        },
        None => Err(error_message(id))
    };
}

impl InputEventTarget for HtmlElement {
    fn set_onmousemove(&self, mut function : Box<dyn FnMut(InputEvent) -> ExpectedUnit>) {
        let closure : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {

            let offset_x = get_property_as_f64(&js_value, "offsetX").unwrap();
            let offset_y = get_property_as_f64(&js_value, "offsetY").unwrap();

            let event = InputEvent::MouseMove { 
                x: offset_x,
                y: offset_y
            };

            function(event).unwrap();
        });

        let closure = Closure::wrap(closure);
        let result = Some(closure.as_ref().unchecked_ref());
        HtmlElement::set_onmousemove(&self, result);
        closure.forget();
    }

    fn set_ontouchstart(&self, mut function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>) {
        let closure : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            function(TouchEvent::try_from(js_value).unwrap()).unwrap();
        });

        let closure = Closure::wrap(closure);
        let result = Some(closure.as_ref().unchecked_ref());
        HtmlElement::set_ontouchstart(&self, result);
        closure.forget();
    }
    
    fn set_ontouchmove(&self, mut function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>) {
        let closure : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            function(TouchEvent::try_from(js_value).unwrap()).unwrap();
        });

        let closure = Closure::wrap(closure);
        let result = Some(closure.as_ref().unchecked_ref());
        HtmlElement::set_ontouchmove(&self, result);
        closure.forget();
    }

    fn set_ontouchend(&self, mut function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>) {
        let closure : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            function(TouchEvent::try_from(js_value).unwrap()).unwrap();
        });

        let closure = Closure::wrap(closure);
        let result = Some(closure.as_ref().unchecked_ref());
        HtmlElement::set_ontouchend(&self, result);
        closure.forget();
    }

    fn set_ontouchcancel(&self, mut function : Box<dyn FnMut(TouchEvent) -> ExpectedUnit>) {
        let closure : Box<dyn FnMut(JsValue)> = Box::new(move |js_value : JsValue| {
            function(TouchEvent::try_from(js_value).unwrap()).unwrap();
        });

        let closure = Closure::wrap(closure);
        let result = Some(closure.as_ref().unchecked_ref());
        HtmlElement::set_ontouchcancel(&self, result);
        closure.forget();
    }
}

pub struct EventQueues {
    pub keyboard_events : Vec<KeyboardEvent>,
    pub mouse_events : Vec<MouseEvent>,
    pub touch_events : Vec<TouchEvent>
}

impl EventQueues {
    pub fn new() -> Rc<RefCell<EventQueues>> {
        let event_queues = EventQueues {
            keyboard_events: Vec::<KeyboardEvent>::new(),
            mouse_events: Vec::<MouseEvent>::new(),
            touch_events: Vec::<TouchEvent>::new()
        };

        return Rc::new(RefCell::new(event_queues))
    }

    pub fn bind_all_queues(event_queues : Weak<RefCell<EventQueues>>, source : &HtmlElement) {
        let closure = Box::new(move |value : TouchEvent| -> ExpectedUnit {
            match event_queues.upgrade() {
                Some(event_queues) => event_queues.borrow_mut().touch_events.push(value),
                None => ()
            }

            Ok(())
        });

        <HtmlElement as InputEventTarget>::set_ontouchstart(source.as_ref(), closure.clone());
        <HtmlElement as InputEventTarget>::set_ontouchmove(source.as_ref(), closure.clone());
        <HtmlElement as InputEventTarget>::set_ontouchend(source.as_ref(), closure.clone());
        <HtmlElement as InputEventTarget>::set_ontouchcancel(source.as_ref(), closure.clone());
    }

    pub fn clear_all_queues(event_queues : &Rc<RefCell<EventQueues>>) {
        let mut event_queues = event_queues.borrow_mut();
        event_queues.keyboard_events.clear();
        event_queues.mouse_events.clear();
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
        let mut id_map : HashMap<u32, usize> = HashMap::new();

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
                    let mut retain_set : HashSet<u32> = HashSet::new();

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
    state : HashSet<KeyCode>
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState { state: HashSet::new() }
    }

    pub fn update_legacy(&mut self, input_events : &Vec<InputEvent>) {
        for event in input_events {
            match event {
                InputEvent::KeyDown { code } => { self.state.insert(*code); },
                InputEvent::KeyUp { code } => { self.state.remove(code); },
                _ => ()
            }
        }
    }

    pub fn is_down(&self, code : KeyCode) -> bool {
       self.state.contains(&code)
    }
}
