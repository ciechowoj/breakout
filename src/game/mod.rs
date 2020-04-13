mod bricks;
mod config;
mod utils;
mod scoreboard;

use glm::*;
use crate::utils::*;
use crate::collision::*;
use crate::dom_utils::*;
use crate::game::bricks::*;
use crate::game::utils::*;
use crate::game::scoreboard::*;
use std::mem::*;
use std::cmp::{max};
use std::include_str;
use wasm_bindgen::prelude::*;
use web_sys::*;
use js_sys::Math::random;

pub fn fmin(a: f32, b: f32) -> f32 { if a < b { a } else { b } }
pub fn fmax(a: f32, b: f32) -> f32 { if a < b { b } else { a } }

pub fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

pub fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    let v_dot_n = dot(&v, &n);
    if v_dot_n < 0.0 { v - 2f32 * v_dot_n * n } else { v }
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

#[allow(dead_code)]
pub fn draw_vector(
    rendering_context : &CanvasRenderingContext2d,
    origin : Vec2,
    target : Vec2,
    color : &'static str) -> Expected<()> {
    rendering_context.begin_path();
    rendering_context.set_line_width(2f64);
    rendering_context.move_to(origin.x as f64, origin.y as f64);
    rendering_context.line_to(target.x as f64, target.y as f64);
    rendering_context.set_stroke_style(&JsValue::from_str(color));
    rendering_context.stroke();
    return Ok(());
}

trait Renderable {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()>;
}

trait Updateable<T> {
    fn update(
        &mut self,
        canvas_size : Vec2,
        elapsed : GameTime) -> Expected<()>;
}

pub struct Bat { pub position : Vec2, pub velocity : Vec2, pub size : Vec2, pub input : Vec2 }

pub struct Ball {
    pub position : Vec2,
    pub velocity : Vec2,
    pub size : f32,
    pub colliding : bool,
    pub freeze_time : Option<f32>
}

impl Ball {
    pub fn new(canvas_size : Vec2) -> Ball {
        let mut ball = Ball {
            position: vec2(0f32, 0f32),
            velocity: vec2(0f32, 0f32),
            size: config::BALL_START_SIZE,
            colliding: false,
            freeze_time: None
        };

        ball.reset_position(canvas_size);

        return ball;
    }

    pub fn reset_position(&mut self, canvas_size : Vec2) {
        let ball_start_position = vec2(config::BALL_START_X, config::BALL_START_Y);
        let ball_start_angle = config::BALL_START_ANGLE * (random() as f32 - 0.5f32);
        let ball_start_direction = rotate_vec2(&vec2(0f32, -1f32), ball_start_angle);

        self.position = mul(ball_start_position, canvas_size);
        self.velocity = ball_start_direction * config::BALL_VELOCITY;

        self.freeze_time = Some(0f32);
    }

    pub fn effective_velocity(&self) -> Vec2 {
        return self.velocity * match self.freeze_time { Some(_) => 0f32, None => 1f32 };
    }
}

impl Renderable for Bat {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()> {
        let origin = self.position - self.size * 0.5;
        rendering_context.set_fill_style(&JsValue::from_str("black"));
        rendering_context.fill_rect(origin.x as f64, origin.y as f64, self.size.x as f64, self.size.y as f64);
        return Ok(());
    }
}

impl Renderable for Ball {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()> {
        let color = match self.freeze_time {
            Some(_) => "grey",
            None => if self.colliding { "red" } else { "black" }
        };

        draw_circle(rendering_context, self.position, self.size, color)?;
        return Ok(());
    }
}

impl Renderable for Brick {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> Expected<()> {
        let size = self.size * (1f32 - fmin(1f32, self.destruction_time.unwrap_or(0f32)));

        let origin = self.position - size * 0.5;

        match self.destruction_time {
            Some(_) => rendering_context.set_fill_style(&JsValue::from_str("red")),
            None => rendering_context.set_fill_style(&JsValue::from_str("black"))
        }

        rendering_context.fill_rect(origin.x as f64, origin.y as f64, size.x as f64, size.y as f64);
        return Ok(());
    }
}

impl Updateable<Bat> for GameState {
    fn update(
        &mut self,
        canvas_size : Vec2,
        game_time : GameTime) -> Expected<()> {

        let bat = &mut self.bat;
        bat.position += mul(bat.input * game_time.elapsed, bat.velocity);

        bat.position.x -= fmin(bat.position.x - bat.size.x * 0.5, 0f32);
        bat.position.x -= fmax(bat.position.x + bat.size.x * 0.5 - canvas_size.x, 0f32);

        return Ok(());
    }
}

impl Updateable<Ball> for GameState {
    fn update(
        &mut self,
        canvas_size : Vec2,
        game_time : GameTime) -> Expected<()> {
        let elapsed = game_time.elapsed;
        let bat = &mut self.bat;
        let ball = &mut self.ball;
        let new_position = ball.position + ball.effective_velocity() * elapsed;
        let mut outer_collision : Option<Collision> = None;

        ball.freeze_time = match ball.freeze_time {
            Some(time) => if time > config::BALL_FREEZE_TIME { None } else { Some(time + elapsed) },
            None => None
        };

        for brick in &mut self.bricks.bricks {
            if let None = brick.destruction_time {
                if let Some(collision) = resolve_circle_aabb_collision(
                    ball.position,
                    new_position, 
                    ball.size,
                    brick.position,
                    brick.size * 0.5) {
                    outer_collision = Some(collision);
                    brick.destruction_time = Some(0f32);
                    self.score += 1;
                }
            }
        }

        if let Some(collision) = resolve_circle_aabb_collision(
            ball.position,
            new_position, 
            ball.size,
            bat.position,
            bat.size * 0.5) {
            outer_collision = Some(collision);
        }

        if let Some(collision) = resolve_circle_aabb_inv_collision(
            ball.position,
            new_position, 
            ball.size,
            canvas_size * 0.5,
            canvas_size * 0.5) {
            if collision.normal != vec2(0f32, -1f32) {
                outer_collision = Some(collision);
            }
        }

        if let Some(collision) = outer_collision {
            let reflected = reflect(ball.velocity, collision.normal);
            ball.position = ball.position + ball.velocity * elapsed * collision.t + reflected * elapsed * (1.0 - collision.t);
            ball.velocity = reflected;
            self.collision = outer_collision.clone();
        }
        else {
            ball.position = new_position;
        }

        if ball.position.y - ball.size > canvas_size.y {
            self.lives = max(self.lives, 1) - 1;

            if self.lives != 0 {
                ball.reset_position(canvas_size);
            }
            else {
                self.stage = GameStage::GameOver;
                self.game_over_time = game_time.real_time;
            }
        }

        return Ok(());
    }
}

impl Updateable<Brick> for GameState {
    fn update(
        &mut self,
        _canvas_size : Vec2,
        game_time : GameTime) -> Expected<()> {

        self.bricks.update(game_time.elapsed)?;

        return Ok(());
    }
}

pub enum GameStage {
    Gameplay,
    GameOver,
    ScoreBoard
}

pub struct GameState {
    pub stage : GameStage,
    pub bat : Bat,
    pub ball : Ball,
    pub bricks : Bricks,
    pub last_time : f64,
    pub time : GameTime,
    pub score : u64,
    pub lives : u32,
    pub game_over_time : f64,
    pub collision : Option<Collision>
}

impl GameState {
    pub fn new(
        bat : Bat,
        ball : Ball,
        bricks : Bricks,
        last_time : f64) -> GameState {
        GameState {
            stage: GameStage::Gameplay,
            bat: bat,
            ball: ball,
            bricks: bricks,
            last_time: last_time,
            time: GameTime { sim_time: 0f64, real_time: 0f64, elapsed: 0f32 },
            collision: None,
            score: 0,
            lives: 1,
            game_over_time : 0f64
        }
    }
}

pub fn init(
    canvas_size : Vec2,
    time : f64) -> GameState {

    let bat_position = vec2(canvas_size.x * 0.5, canvas_size.y - 100.0);

    let bat = Bat { 
        position: bat_position,
        velocity: vec2(1000.0, 1000.0),
        size: vec2(200.0, 20.0),
        input: vec2(0.0, 0.0)
    };

    let mut ball = Ball::new(canvas_size);

    ball.reset_position(canvas_size);

    let bricks = Bricks::new(canvas_size);

    GameState::new(bat, ball, bricks, time)
}

pub fn update_bat_input(
    game_state : &mut GameState,
    expected_input: Option<Vec2>,
    new_input : Vec2) {
    let bat = &mut game_state.bat;

    if expected_input.is_none() 
    || bat.input.x == expected_input.unwrap().x 
    && bat.input.y == expected_input.unwrap().y {
        bat.input = new_input;
    }
}

pub fn init_overlay(
    _game_state : &mut GameState,
    overlay : &HtmlElement,
    _time : f64) -> Expected<()> {

    let document = overlay.owner_document().ok_or(Error::Msg("Failed to get document node."))?;
    create_style_element(&document, include_str!("game.css"), "game-css")?;

    let score = create_html_element(&document, "span", "footer-score")?;
    score.set_inner_html("0");
    overlay.append_child(&score)?;

    let lives = create_html_element(&document, "span", "footer-lives")?;
    lives.set_inner_html("");
    overlay.append_child(&lives)?;

    return Ok(());
}

pub fn update_game_over(
    document : &Document,
    game_state : &mut GameState,
    overlay : &HtmlElement) -> Expected<()> {

    let game_over_id = "game-over";
    let game_over = try_get_html_element_by_id(document, game_over_id)?;

    match game_over {
        Some(element) => {
            match game_state.stage {
                GameStage::GameOver => {},
                _ => {
                    overlay.remove_child(&element)?;
                }
            }
        },
        None => {
            match game_state.stage {
                GameStage::GameOver => {
                    let game_over = create_html_element(&document, "div", game_over_id)?;
                    game_over.set_inner_html("<span>Game Over</span>");
                    overlay.append_child(&game_over)?;
                },
                _ => {}
            }
        }
    };

    return Ok(());
}

pub fn update_score_board(
    document : &Document,
    game_state : &mut GameState,
    overlay : &HtmlElement) -> Expected<()> {
    let score_board_id = "score-board";
    let score_board = try_get_html_element_by_id(document, score_board_id)?;

    match score_board {
        Some(element) => {
            match game_state.stage {
                GameStage::ScoreBoard => {},
                _ => {
                    overlay.remove_child(&element)?;
                }
            }
        },
        None => {
            match game_state.stage {
                GameStage::ScoreBoard => {
                    create_scoreboard(&document, &overlay, game_state.score)?;
                },
                _ => {}
            }
        }
    };

    return Ok(());
}

pub fn update_overlay(
    game_state : &mut GameState,
    overlay : &HtmlElement,
    _time : f64) -> Expected<()> {

    let document = overlay.owner_document().ok_or(Error::Msg("Failed to get document node."))?;

    let score = get_html_element_by_id(&document, "footer-score")?;
    let lives = get_html_element_by_id(&document, "footer-lives")?;

    match game_state.stage {
        GameStage::Gameplay | GameStage::GameOver => {
            let score_str = game_state.score.to_string();
            score.style().remove_property("display")?;
            score.set_inner_html(&score_str[..]);

            let lives_str =  "❤".repeat(game_state.lives as usize);
            lives.style().remove_property("display")?;
            lives.set_inner_html(&lives_str[..]);
        },
        _ => {
            score.style().set_property("display", "none")?;
            lives.style().set_property("display", "none")?;
        }
    };

    update_game_over(&document, game_state, overlay)?;
    update_score_board(&document, game_state, overlay)?;

    return Ok(());
}

pub fn update(
    game_state : &mut GameState,
    input_events : &Vec<InputEvent>,
    canvas_size : Vec2,
    time : f64) -> Expected<()> {

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
            InputEvent::KeyUp { time: _, code } => {
                match code {
                    KeyCode::ArrowLeft => { update_bat_input(game_state, Some(vec2(-1.0, 0.0)), vec2(0.0, 0.0)) }
                    KeyCode::ArrowRight => { update_bat_input(game_state, Some(vec2(1.0, 0.0)), vec2(0.0, 0.0)) }
                    KeyCode::Space => { game_state.bricks.reset_last_row() }
                    KeyCode::Enter => {   
                        match game_state.stage {
                            GameStage::ScoreBoard => {
                                *game_state = init(canvas_size, time);
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }

                log!("{} key released at time {:.2}!", code.as_ref(), time);
            },
            InputEvent::MouseMove { time: _, x, y } => {
                // game_state.ball.position.x = *x as f32;
                // game_state.ball.position.y = *y as f32;

                log!("Mouse moved to position {:.2} {:.2} at time {:.2}!", x, y, time);
            }
        }
    }

    let epsilon = 0.01f64;
    let mut current = game_state.last_time;
    game_state.time.real_time = time;
    
    while epsilon < time - current {
        game_state.time.sim_time += epsilon as f64;
        game_state.time.elapsed = epsilon as f32;

        match game_state.stage {
            GameStage::Gameplay => {
                Updateable::<Ball>::update(game_state, canvas_size, game_state.time)?;
                Updateable::<Bat>::update(game_state, canvas_size, game_state.time)?;
            },
            GameStage::GameOver => {
                if game_state.time.real_time - game_state.game_over_time > config::GAME_OVER_PAUSE_TIME {
                    game_state.stage = GameStage::ScoreBoard
                }
            }
            _ => {}
        };
        
        Updateable::<Brick>::update(game_state, canvas_size, game_state.time)?;
        current += epsilon as f64;
    }

    game_state.last_time = current;

    return Ok(());
}

pub fn render(
    game_state : &GameState,
    rendering_context : &CanvasRenderingContext2d,
    canvas_size : Vec2,
    _time : f64) -> Expected<()> {
    let width = canvas_size.x as f64;
    let height = canvas_size.y as f64;

    rendering_context.set_fill_style(&JsValue::from_str("lightgray"));
    rendering_context.fill_rect(0.0, 0.0, width, height);

    match game_state.stage {
        GameStage::Gameplay | GameStage::GameOver => {
            for entity in &game_state.bricks.bricks {
                entity.render(rendering_context)?;
            }
            
            game_state.ball.render(rendering_context)?;
        },
        _ => ()
    };
    
    match game_state.stage {
        GameStage::Gameplay => game_state.bat.render(rendering_context)?,
        _ => ()
    };

    /* if let Some(collision) = &game_state.collision {
        draw_vector(rendering_context, collision.point, collision.point + collision.normal * 32f32, "green")?;
        draw_circle(rendering_context, collision.point, 3.0, "green")?;
        draw_vector(rendering_context, collision.point, collision.point + game_state.ball.velocity, "blue")?;
        let reflected = reflect(-game_state.ball.velocity, collision.normal) * (1.0 - collision.t);
        draw_vector(rendering_context, collision.point, collision.point + reflected, "yellow")?;
    }*/

    return Ok(());
}
