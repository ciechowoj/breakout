mod bat;
mod ball;
mod bricks;
mod config;
mod utils;
mod scoreboard;

use glm::*;
use crate::event::*;
use crate::utils::*;
use crate::dom_utils::*;
use crate::game::bricks::*;
use crate::game::bat::*;
use crate::game::ball::*;
use crate::game::utils::*;
use crate::game::scoreboard::*;
use std::cmp::{max};
use std::include_str;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::*;

pub fn draw_circle(
    rendering_context : &CanvasRenderingContext2d,
    origin : Vec2,
    radius : f32,
    color : &'static str) -> anyhow::Result<()> {
    rendering_context.begin_path();
    rendering_context.arc(origin.x as f64, origin.y as f64, radius as f64, 0.0, two_pi()).to_anyhow()?;
    rendering_context.set_fill_style(&JsValue::from_str(color));
    rendering_context.fill();
    return Ok(());
}

#[allow(dead_code)]
pub fn draw_vector(
    rendering_context : &CanvasRenderingContext2d,
    origin : Vec2,
    target : Vec2,
    color : &'static str) -> anyhow::Result<()> {
    rendering_context.begin_path();
    rendering_context.set_line_width(2f64);
    rendering_context.move_to(origin.x as f64, origin.y as f64);
    rendering_context.line_to(target.x as f64, target.y as f64);
    rendering_context.set_stroke_style(&JsValue::from_str(color));
    rendering_context.stroke();
    return Ok(());
}

trait Renderable {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> anyhow::Result<()>;
}

impl Renderable for Bat {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> anyhow::Result<()> {
        let origin = self.position - self.size * 0.5;
        rendering_context.set_fill_style(&JsValue::from_str("black"));
        rendering_context.fill_rect(origin.x as f64, origin.y as f64, self.size.x as f64, self.size.y as f64);
        return Ok(());
    }
}

impl Renderable for Ball {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> anyhow::Result<()> {
        let color = match self.freeze_time {
            Some(_) => "grey",
            None => if self.colliding { "red" } else { "black" }
        };

        draw_circle(rendering_context, self.position, self.size, color)?;
        return Ok(());
    }
}

impl Renderable for Brick {
    fn render(&self, rendering_context : &CanvasRenderingContext2d) -> anyhow::Result<()> {
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

fn decrease_lives(game_state : &mut GameState, canvas_size : Vec2, game_time : GameTime) {
    let ball : &mut Ball = &mut game_state.ball;

    game_state.lives = max(game_state.lives, 1) - 1;

    if game_state.lives != 0 {
        ball.reset_position(canvas_size);
    }
    else {
        game_state.stage = GameStage::GameOver;
        game_state.game_over_time = game_time.real_time;
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
    pub score : i64,
    pub score_id : Rc<RefCell<uuid::Uuid>>,
    pub lives : u32,
    pub game_over_time : f64,
    pub keyboard_state : KeyboardState,
    pub touch_tracker : TouchTracker
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
            score: 4001,
            score_id: Rc::new(RefCell::new(uuid::Uuid::nil())),
            lives: 1,
            game_over_time: 0f64,
            keyboard_state: KeyboardState::new(),
            touch_tracker: TouchTracker::new()
        }
    }
}

pub fn init(
    canvas_size : Vec2,
    time : f64) -> GameState {

    let bat = Bat::new(canvas_size);
    let mut ball = Ball::new(canvas_size);

    ball.reset_position(canvas_size);

    let bricks = Bricks::new(canvas_size);

    GameState::new(bat, ball, bricks, time)
}

pub fn init_overlay(
    _game_state : &mut GameState,
    overlay : &HtmlElement,
    _time : f64) -> anyhow::Result<()> {

    let document = overlay.owner_document().ok_or(anyhow::anyhow!("Failed to get document node."))?;
    create_style_element(&document, include_str!("game.css"), "game-css")?;

    let score = create_html_element(&document, "span", "footer-score")?;
    score.set_inner_html("0");
    overlay.append_child(&score).to_anyhow()?;

    let lives = create_html_element(&document, "span", "footer-lives")?;
    lives.set_inner_html("");
    overlay.append_child(&lives).to_anyhow()?;

    return Ok(());
}

pub fn update_game_over(
    game_state : &mut GameState,
    overlay : &HtmlElement) -> anyhow::Result<()> {

    let document = overlay
        .owner_document().ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let game_over_id = "game-over";
    let game_over = try_get_html_element_by_id(&document, game_over_id)?;

    match game_over {
        Some(element) => {
            match game_state.stage {
                GameStage::GameOver => {},
                _ => {
                    overlay.remove_child(&element).to_anyhow()?;
                }
            }
        },
        None => {
            match game_state.stage {
                GameStage::GameOver => {
                    let game_over = create_html_element(&document, "div", game_over_id)?;
                    game_over.set_inner_html("<span>Game Over</span>");
                    overlay.append_child(&game_over).to_anyhow()?;
                },
                _ => {}
            }
        }
    };

    return Ok(());
}

pub fn update_score_board(
    game_state : &mut GameState,
    overlay : &HtmlElement) -> anyhow::Result<()> {

    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score_board_id = "score-board";
    let score_board = try_get_html_element_by_id(&document, score_board_id)?;

    match score_board {
        Some(element) => {
            match game_state.stage {
                GameStage::ScoreBoard => {},
                _ => {
                    overlay.remove_child(&element).to_anyhow()?;
                }
            }
        },
        None => {
            match game_state.stage {
                GameStage::ScoreBoard => {
                    create_scoreboard(overlay.clone(), game_state.score, game_state.score_id.clone(), score_board_id)?;
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
    _time : f64) -> anyhow::Result<()> {

    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score = get_html_element_by_id(&document, "footer-score")?;
    let lives = get_html_element_by_id(&document, "footer-lives")?;

    match game_state.stage {
        GameStage::Gameplay | GameStage::GameOver => {
            let score_str = game_state.score.to_string();
            score.style().remove_property("display").to_anyhow()?;
            score.set_inner_html(&score_str[..]);

            let lives_str =  "❤".repeat(game_state.lives as usize);
            lives.style().remove_property("display").to_anyhow()?;
            lives.set_inner_html(&lives_str[..]);
        },
        _ => {
            score.style().set_property("display", "none").to_anyhow()?;
            lives.style().set_property("display", "none").to_anyhow()?;
        }
    };

    update_game_over(game_state, overlay)?;
    update_score_board(game_state, overlay)?;

    return Ok(());
}

pub fn update(
    game_state : &mut GameState,
    overlay : &HtmlElement,
    input_events : &Vec<InputEvent>,
    event_queues : &EventQueues,
    canvas_size : Vec2,
    time : f64) -> anyhow::Result<()> {

    for event in input_events {
        match event {
            InputEvent::KeyDown { code } => {
                match code {
                    _ => {}
                }

                log!("{} key pressed at time {:.2}!", code.as_ref(), time);
            },
            InputEvent::KeyUp { code } => {
                match code {
                    KeyCode::Space => { game_state.bricks.reset_last_row() }
                    KeyCode::Enter => {
                        match game_state.stage {
                            GameStage::ScoreBoard => {
                                if let Some(name) = player_name()? {
                                    persist_score(overlay.clone(), name, *game_state.score_id.borrow())?;
                                }

                                *game_state = init(canvas_size, time);
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }

                log!("{} key released at time {:.2}!", code.as_ref(), time);
            },
            InputEvent::MouseMove { x : _, y : _ } => { }
        }
    }

    game_state.keyboard_state.update_legacy(input_events);
    game_state.touch_tracker.update(&event_queues.touch_events);

    let left_arrow = game_state.keyboard_state.is_down(KeyCode::ArrowLeft);
    let right_arrow = game_state.keyboard_state.is_down(KeyCode::ArrowRight);

    game_state.bat.input = vec2(0f32, 0f32);

    if left_arrow || right_arrow {
        if left_arrow {
            game_state.bat.input = vec2(-1f32, 0f32);
        }
        else {
            game_state.bat.input = vec2(1f32, 0f32);
        }
    }
    else {
        for touch in &game_state.touch_tracker.touches {
            if touch.client_x < 500f32 {
                game_state.bat.input = vec2(-1f32, 0f32);
            }
            else {
                game_state.bat.input = vec2(1f32, 0f32);
            }
        }
    }

    for touch in &game_state.touch_tracker.touches {
        log!("{:?}", touch);
    }

    let epsilon = 0.01f64;
    let mut current = game_state.last_time;
    game_state.time.real_time = time;

    while epsilon < time - current {
        game_state.time.sim_time += epsilon as f64;
        game_state.time.elapsed = epsilon as f32;

        match game_state.stage {
            GameStage::Gameplay => {
                let ball_status = update_ball(&game_state.bat, &mut game_state.ball, &mut game_state.bricks, canvas_size, game_state.time.elapsed)?;

                game_state.score += ball_status.brick_hit_count as i64;

                if ball_status.out_of_arena {
                    decrease_lives(game_state, canvas_size, game_state.time);
                }

                update_bat(&mut game_state.bat, canvas_size, game_state.time.elapsed)?;
            },
            GameStage::GameOver => {
                if game_state.time.real_time - game_state.game_over_time > config::GAME_OVER_PAUSE_TIME {
                    game_state.stage = GameStage::ScoreBoard
                }
            }
            _ => {}
        };

        game_state.bricks.update(game_state.time.elapsed)?;
        current += epsilon as f64;
    }

    game_state.last_time = current;

    return Ok(());
}

pub fn render(
    game_state : &GameState,
    rendering_context : &CanvasRenderingContext2d,
    canvas_size : Vec2,
    _time : f64) -> anyhow::Result<()> {
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

    return Ok(());
}
