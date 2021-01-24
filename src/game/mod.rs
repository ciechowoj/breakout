mod bat;
mod ball;
mod bricks;
mod config;
pub mod utils;
mod scoreboard;

use glm::*;
use crate::event::*;
use crate::utils::*;
use crate::game::bricks::*;
use crate::game::bat::*;
use crate::game::ball::*;
use crate::game::utils::*;
use crate::game::scoreboard::*;
use std::cmp::{max};
use std::include_str;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::DerefMut;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast};
use web_sys::*;

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

fn decrease_lives(game_state : &mut GameState, game_time : GameTime) {
    let ball : &mut Ball = &mut game_state.ball;

    game_state.lives = max(game_state.lives, 1) - 1;

    if game_state.lives != 0 {
        ball.reset_position();
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
    pub keyboard_state : Rc<RefCell<KeyboardState>>,
    pub touch_tracker : TouchTracker,
    pub reset_requested : bool
}

impl GameState {
    pub fn new(
        bat : Bat,
        ball : Ball,
        bricks : Bricks,
        last_time : f64) -> Rc<RefCell<GameState>> {
        let game_state = GameState {
            stage: GameStage::Gameplay,
            bat: bat,
            ball: ball,
            bricks: bricks,
            last_time: last_time,
            time: GameTime { sim_time: 0f64, real_time: 0f64, elapsed: 0f32 },
            score: 0,
            score_id: Rc::new(RefCell::new(uuid::Uuid::nil())),
            lives: 3,
            game_over_time: 0f64,
            keyboard_state: KeyboardState::new(),
            touch_tracker: TouchTracker::new(),
            reset_requested: false
        };

        let game_state = Rc::new(RefCell::new(game_state));
        GameState::register_event_listeners(&game_state);
        return game_state;
    }

    fn register_event_listeners(game_state : &Rc<RefCell<GameState>>) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let game_state = game_state.clone();

        let on_keyup : Box<dyn FnMut(JsValue)> = {
            let document = document.clone();

            Box::new(move |js_value : JsValue| {
                let code = get_code(js_value).unwrap();

                let game_state : &mut GameState = &mut game_state.borrow_mut();

                match code {
                    KeyCode::Enter => {
                        match game_state.stage {
                            GameStage::ScoreBoard => {
                                if !game_state.reset_requested {
                                    if let Some(name) = player_name().unwrap() {
                                        let overlay : HtmlElement = document.get_element_by_id("main-overlay").unwrap().unchecked_into();
                                        let message = "Are you sure you want to post you score and nickname? The record cannot be changed or removed.";

                                        if window.confirm_with_message(message).unwrap() {
                                            persist_score(overlay, name, *game_state.score_id.borrow()).unwrap();
                                        }
                                    }

                                    game_state.reset_requested = true;
                                }
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            })
        };

        let closure = Closure::wrap(on_keyup as Box<dyn FnMut(JsValue)>);
        document.add_event_listener_with_callback("keyup", closure.as_ref()
            .unchecked_ref()).unwrap();
        closure.forget();
    }

    pub fn init(time : f64) -> Rc<RefCell<GameState>> {
        let bat = Bat::new();
        let mut ball = Ball::new();

        ball.reset_position();

        let bricks = Bricks::new();

        return GameState::new(bat, ball, bricks, time);
    }
}

pub fn init_overlay(
    _game_state : &mut GameState,
    overlay : &HtmlElement,
    _time : f64) -> anyhow::Result<()> {

    let document = overlay.owner_document().ok_or(anyhow::anyhow!("Failed to get document node."))?;
    utils::create_style_element(&document, include_str!("game.css"), "game-css")?;

    let score : HtmlElement = document.create_element("span").unwrap().unchecked_into();
    score.set_id("footer-score");
    score.set_inner_html("0");
    overlay.append_child(&score).to_anyhow()?;

    let lives : HtmlElement = document.create_element("span").unwrap().unchecked_into();
    lives.set_id("footer-lives");
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
    let game_over = document.get_element_by_id(game_over_id);

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
                    let game_over : HtmlElement = document.create_element("div").unwrap().unchecked_into();
                    game_over.set_id(game_over_id);
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
    let score_board = document.get_element_by_id(score_board_id);

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

    let score : HtmlElement = document.get_element_by_id("footer-score").unwrap().unchecked_into();
    let lives : HtmlElement = document.get_element_by_id("footer-lives").unwrap().unchecked_into();

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
    game_state_rc : &mut Rc<RefCell<GameState>>,
    event_queues : &EventQueues,
    time : f64) -> anyhow::Result<()> {

    if game_state_rc.borrow().reset_requested {
        *game_state_rc = GameState::init(time);
    }

    {
        let mut borrow_mut = game_state_rc.borrow_mut();
        let mut game_state : &mut GameState = borrow_mut.deref_mut();

        game_state.touch_tracker.update(&event_queues.touch_events);

        let left_arrow = game_state.keyboard_state.borrow().is_down(KeyCode::ArrowLeft);
        let right_arrow = game_state.keyboard_state.borrow().is_down(KeyCode::ArrowRight);

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
                    let ball_status = update_ball(&game_state.bat, &mut game_state.ball, &mut game_state.bricks, game_state.time.elapsed)?;

                    game_state.score += ball_status.brick_hit_count as i64;

                    if ball_status.out_of_arena {
                        decrease_lives(game_state, game_state.time);
                    }

                    update_bat(&mut game_state.bat, game_state.time.elapsed)?;
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
    }

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
    rendering_context.set_transform(
        width / config::GAME_AREA_WIDTH,
        0.0,
        0.0,
        height / config::GAME_AREA_HEIGHT,
        0.0,
        0.0).unwrap();



    match game_state.stage {
        GameStage::Gameplay | GameStage::GameOver => {
            for entity in &game_state.bricks.bricks {
                render_brick(entity, rendering_context)?;
            }

            render_ball(&game_state.ball, rendering_context)?;
        },
        _ => ()
    };

    match game_state.stage {
        GameStage::Gameplay => render_bat(&game_state.bat, rendering_context)?,
        _ => ()
    };

    rendering_context.reset_transform().unwrap();

    return Ok(());
}
