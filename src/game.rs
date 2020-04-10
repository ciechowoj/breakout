use crate::vec2::*;
use crate::utils::*;
use std::mem::*;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

trait Renderable {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()>;
}

pub enum GameEntity {
    Bat { position : vec2, size : vec2 },
    Ball { position : vec2, size : f32 },
    Brick { position : vec2, size : vec2 }
}

pub struct GameState {
    entities : Vec<GameEntity>,
    pub x : f32, pub y : f32, pub last_time : f64,
    pub x_speed : f32, pub y_speed : f32
}

pub fn update(
    game_state : &mut GameState,
    input_events : &Vec<InputEvent>,
    canvas_size : vec2,
    time : f64) -> () {

    let elapsed = time - game_state.last_time;
    
    for event in input_events {
        match event {
            InputEvent::KeyDown { time, code } => {
                log!("{} key pressed at time {:.2}!", code.as_ref(), time);
                log!("x: {}, y: {}", game_state.x, game_state.y);
                log!("{}", size_of::<JsValue>());
            },
            InputEvent::KeyUp { time, code } => {
                log!("{} key released at time {:.2}!", code.as_ref(), time);
            }
        }
    }

    game_state.x += game_state.x_speed * elapsed as f32;
    game_state.y += game_state.y_speed * elapsed as f32;

    if game_state.x > canvas_size.x || game_state.x < 0.0 {
        game_state.x_speed *= -1.0;
    }

    if game_state.y > canvas_size.y as f32 || game_state.y < 0.0 {
        game_state.y_speed *= -1.0;
    }

    game_state.last_time = time;
}

impl Renderable for GameEntity {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()> {

        match self {
            GameEntity::Bat { position, size } => {
                rendering_context
                    .set_fill_style(&JsValue::from_str("black"));

                let origin = position - size * 0.5;

                rendering_context
                    .fill_rect(origin.x as f64, origin.y as f64, size.x as f64, size.y as f64);
            }


        }

        return Ok(());
    }
}


pub fn render(
    game_state : &GameState,
    rendering_context : &CanvasRenderingContext2d,
    canvas_size : vec2,
    time : f64) -> Expected<()> {
    let width = canvas_size.x as f64;
    let height = canvas_size.y as f64;

    rendering_context.clear_rect(0.0, 0.0, width, height);
    rendering_context.set_fill_style(&JsValue::from_str("black"));
    rendering_context.fill_rect((width - 100.0) * (time * 0.1).fract(), 10.0, 100.0, 100.0);

    rendering_context.fill_rect(0.0, 200.0, width, 100.0);

    rendering_context.begin_path();

    rendering_context.arc(game_state.x as f64, game_state.y as f64, 10.0, 0.0, 2.0 * 3.14)?;

    rendering_context.set_fill_style(&JsValue::from_str("green"));
    rendering_context.fill();

    return Ok(());
}
