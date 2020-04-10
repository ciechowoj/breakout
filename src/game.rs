use glm::*;
use crate::utils::*;
use crate::collision::*;
use std::mem::*;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

pub fn draw_circle(
    rendering_context : &CanvasRenderingContext2d,
    origin : Vec2,
    radius : f32,
    color : &'static str) -> Expected<()> {
    rendering_context.begin_path();
    rendering_context.arc(origin.x as f64, origin.y as f64, radius as f64, 0.0, two_pi())?;
    rendering_context.set_fill_style(&JsValue::from_str(color));
    rendering_context.fill();
    return Ok(());
}

trait Renderable {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()>;
}

trait Updateable {
    fn update(&mut self, canvas_size : Vec2, elapsed : f32) -> Expected<()>;
}

pub enum GameEntity {
    Bat { position : Vec2, velocity : Vec2, size : Vec2, input : Vec2 },
    Ball { position : Vec2, velocity : Vec2, size : f32 },
    Brick { position : Vec2, size : Vec2 }
}

impl Renderable for GameEntity {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()> {

        match self {
            GameEntity::Bat { position, velocity, size, input } => {
                let origin = position - size * 0.5;
                rendering_context.set_fill_style(&JsValue::from_str("black"));
                rendering_context.fill_rect(origin.x as f64, origin.y as f64, size.x as f64, size.y as f64);
            },
            GameEntity::Ball { position, velocity, size } => {
                draw_circle(rendering_context, *position, *size, "black")?;
            },
            GameEntity::Brick { position, size } => {
                let origin = position - size * 0.5;
                rendering_context.set_fill_style(&JsValue::from_str("black"));
                rendering_context.fill_rect(origin.x as f64, origin.y as f64, size.x as f64, size.y as f64);
            }
        }

        return Ok(());
    }
}

impl Updateable for GameEntity {
    fn update(
        &mut self,
        canvas_size : Vec2,
        elapsed : f32) -> Expected<()> {

        match self {
            GameEntity::Bat { position, velocity, size, input } => {
                *position += mul(*input * elapsed, *velocity);
            },
            GameEntity::Ball { position, velocity, size } => {
                /* *position += *velocity * elapsed;

                if position.x > canvas_size.x || position.x < 0.0 {
                    velocity.x *= -1.0;
                }

                if position.y > canvas_size.y as f32 || position.y < 0.0 {
                    velocity.y *= -1.0;
                } */
            },
            GameEntity::Brick { position, size } => {
            }
        }

        return Ok(());
    }
}

pub struct GameState {
    pub entities : Vec<GameEntity>,
    pub last_time : f64,
    pub bat_index : usize
}

impl GameState {
    pub fn new(entities : Vec<GameEntity>, bat_index : usize, last_time : f64) -> GameState {
        GameState {
            entities: entities,
            last_time: last_time,
            bat_index: bat_index
        }
    }
}

pub fn init(
    canvas_size : Vec2,
    time : f64) -> GameState {

    let bat_position = vec2(canvas_size.x * 0.5, canvas_size.y - 100.0);

    let bat = GameEntity::Bat { 
        position: bat_position,
        velocity: vec2(200.0, 200.0),
        size: vec2(100.0, 20.0),
        input: vec2(0.0, 0.0)
    };

    let ball = GameEntity::Ball {
        position: bat_position - vec2(0.0, 50.0),
        velocity: vec2(100.0, 100.0),
        size: 10.0
    };

    let mut entities = vec![bat, ball];

    let bricks_cols = 10;
    let bricks_rows = 5;
    let brick_size = vec2(64f32, 32f32);
    let brick_spacing = vec2(8f32, 8f32);

    let bricks_size = vec2(
        bricks_cols as f32 * (brick_size.x + brick_spacing.x) - brick_spacing.x,
        bricks_rows as f32 * (brick_size.y + brick_spacing.y) - brick_spacing.y
    );

    let bricks_origin = bat_position - vec2(0.0, 400.0) - bricks_size * 0.5;
    let brick_origin = brick_size * 0.5;

    for y in 0..bricks_rows {
        for x in 0..bricks_cols {
            let index = vec2(x as f32, y as f32);
            
            let brick = GameEntity::Brick {
                position: bricks_origin + mul(brick_size + brick_spacing, index) + brick_origin,
                size: brick_size
            };

            entities.push(brick);
        }
    }

    GameState ::new(entities, 0, time)
}

pub fn update_bat_input(
    game_state : &mut GameState,
    expected_input: Option<Vec2>,
    new_input : Vec2) {
    match &mut game_state.entities[game_state.bat_index] {
        GameEntity::Bat { position, velocity, size, input } => { 
            if expected_input.is_none() 
            || input.x == expected_input.unwrap().x && input.y == expected_input.unwrap().y {
                *input = new_input;
            }
        },
        _ => { panic!("No bat at bat index!") }
    };
}

pub fn update(
    game_state : &mut GameState,
    input_events : &Vec<InputEvent>,
    canvas_size : Vec2,
    time : f64) -> Expected<()> {

    let elapsed = time - game_state.last_time;

    for event in input_events {
        match event {
            InputEvent::KeyDown { time, code } => {
                match code {
                    KeyCode::ArrowLeft => { update_bat_input(game_state, None, vec2(-1.0, 0.0)) }
                    KeyCode::ArrowRight => { update_bat_input(game_state, None, vec2(1.0, 0.0)) }
                    _ => {}
                }

                log!("{} key pressed at time {:.2}!", code.as_ref(), time);
                log!("{}", size_of::<JsValue>());
            },
            InputEvent::KeyUp { time, code } => {
                match code {
                    KeyCode::ArrowLeft => { update_bat_input(game_state, Some(vec2(-1.0, 0.0)), vec2(0.0, 0.0)) }
                    KeyCode::ArrowRight => { update_bat_input(game_state, Some(vec2(1.0, 0.0)), vec2(0.0, 0.0)) }
                    _ => {}
                }

                log!("{} key released at time {:.2}!", code.as_ref(), time);
            },
            InputEvent::MouseMove { time, x, y } => {

                for entity in &mut game_state.entities {
                    match entity {
                        GameEntity::Ball { position, velocity, size } =>
                        { 
                            position.x = *x as f32;
                            position.y = *y as f32;
                        }
                        _ => {}
                    }

                    entity.update(canvas_size, elapsed as f32)?;
                }

                log!("Mouse moved to position {:.2} {:.2} at time {:.2}!", x, y, time);
            }
        }
    }

    for entity in &mut game_state.entities {
        entity.update(canvas_size, elapsed as f32)?;
    }

    game_state.last_time = time;

    return Ok(());
}

pub fn render(
    game_state : &GameState,
    rendering_context : &CanvasRenderingContext2d,
    canvas_size : Vec2,
    time : f64) -> Expected<()> {
    let width = canvas_size.x as f64;
    let height = canvas_size.y as f64;

    rendering_context.set_fill_style(&JsValue::from_str("lightgray"));
    rendering_context.fill_rect(0.0, 0.0, width, height);

    for entity in &game_state.entities {
        entity.render(rendering_context)?;
    }

    return Ok(());
}
