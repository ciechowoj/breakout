use glm::*;
use crate::utils::*;
use crate::collision::*;
use std::mem::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::*;

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
        elapsed : f32) -> Expected<()>;
}

pub struct Bat { pub position : Vec2, pub velocity : Vec2, pub size : Vec2, pub input : Vec2 }

pub struct Ball {
    pub position : Vec2,
    pub velocity : Vec2,
    pub size : f32,
    pub colliding : bool
}

impl Ball {
    pub fn new(
        position : Vec2,
        velocity : Vec2,
        size : f32) -> Ball {
        Ball {
            position: position,
            velocity: velocity,
            size: size,
            colliding: false
        }
    }
}

pub struct Brick { 
    pub position : Vec2, 
    pub size : Vec2,
    pub destruction_time : Option<f32>,
}

impl Brick {
    pub fn new(
        position : Vec2,
        size : Vec2) -> Brick {
        Brick {
            position: position,
            size: size,
            destruction_time: None
        }
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
        draw_circle(rendering_context, self.position, self.size, if self.colliding { "red" } else { "black" })?;
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
        elapsed : f32) -> Expected<()> {

        let bat = &mut self.bat;
        bat.position += mul(bat.input * elapsed, bat.velocity);

        bat.position.x -= fmin(bat.position.x - bat.size.x * 0.5, 0f32);
        bat.position.x -= fmax(bat.position.x + bat.size.x * 0.5 - canvas_size.x, 0f32);

        return Ok(());
    }
}

impl Updateable<Ball> for GameState {
    fn update(
        &mut self,
        canvas_size : Vec2,
        elapsed : f32) -> Expected<()> {
        
        let bat = &mut self.bat;
        let ball = &mut self.ball;
        let new_position = ball.position + ball.velocity * elapsed;
        let mut outer_collision : Option<Collision> = None;

        for brick in &mut self.bricks {
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
            outer_collision = Some(collision);
        }

        if let Some(collision) = outer_collision {
            let reflected = reflect(ball.velocity, collision.normal);
            ball.position = ball.position + ball.velocity * elapsed * collision.t + reflected * elapsed* (1.0 - collision.t);
            ball.velocity = reflected;
            self.collision = outer_collision.clone();
        }
        else {
            ball.position = new_position;
        }

        return Ok(());
    }
}

impl Updateable<Brick> for GameState {
    fn update(
        &mut self,
        _canvas_size : Vec2,
        elapsed : f32) -> Expected<()> {

        for brick in &mut self.bricks {
            if let Some(destruction_time) = brick.destruction_time {
                brick.destruction_time = Some(destruction_time + elapsed);
            }
        }

        return Ok(());
    }
}

pub struct GameState {
    pub bat : Bat,
    pub ball : Ball,
    pub bricks : Vec<Brick>,
    pub last_time : f64,
    pub score : u32,
    pub collision : Option<Collision>
}

impl GameState {
    pub fn new(
        bat : Bat,
        ball : Ball,
        bricks : Vec<Brick>,
        last_time : f64) -> GameState {
        GameState {
            bat: bat,
            ball: ball,
            bricks: bricks,
            last_time: last_time,
            collision: None,
            score: 0
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

    let ball = Ball::new(
        bat_position - vec2(0.0, 50.0),
        // vec2(0.0, 0.0), 
        vec2(1000.0, 1000.0),
        19.0);

    let mut bricks : Vec<Brick> = vec![];

    let bricks_cols = 10;
    let bricks_rows = 5;
    let brick_size = vec2(80f32, 40f32);
    let brick_spacing = vec2(20f32, 20f32);

    let bricks_size = vec2(
        bricks_cols as f32 * (brick_size.x + brick_spacing.x) - brick_spacing.x,
        bricks_rows as f32 * (brick_size.y + brick_spacing.y) - brick_spacing.y
    );

    let bricks_origin = bat_position - vec2(0.0, 800.0) - bricks_size * 0.5;
    let brick_origin = brick_size * 0.5;

    for y in 0..bricks_rows {
        for x in 0..bricks_cols {
            let index = vec2(x as f32, y as f32);
            
            let brick = Brick::new(
                bricks_origin + mul(brick_size + brick_spacing, index) + brick_origin,
                brick_size
            );

            bricks.push(brick);
        }
    }

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

    let score = document.create_element("span")?;
    let score = score.dyn_into::<web_sys::HtmlElement>()
        .map_err(|_| ())
        .unwrap();
    
    score.set_id("score");
    score.style().set_property("position", "absolute")?;
    score.style().set_property("bottom", "0px")?;
    score.style().set_property("width", "100%")?;
    score.style().set_property("text-align", "center")?;
    score.style().set_property("font-family", "Helvetica, Arial, sans-serif")?;
    score.style().set_property("font-size", "48px")?;

    score.set_inner_html("0");
    overlay.append_child(&score)?;

    return Ok(());
}

pub fn update_overlay(
    game_state : &mut GameState,
    overlay : &HtmlElement,
    _time : f64) -> Expected<()> {

    let document = overlay.owner_document().ok_or(Error::Msg("Failed to get document node."))?;

    if let Some(score) = document.get_element_by_id("score") {
        let score = score.dyn_into::<web_sys::HtmlElement>()
            .map_err(|_| ())
            .unwrap();

        let score_str = game_state.score.to_string();
        score.set_inner_html(&score_str[..]);
    }

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
            InputEvent::KeyUp { time, code } => {
                match code {
                    KeyCode::ArrowLeft => { update_bat_input(game_state, Some(vec2(-1.0, 0.0)), vec2(0.0, 0.0)) }
                    KeyCode::ArrowRight => { update_bat_input(game_state, Some(vec2(1.0, 0.0)), vec2(0.0, 0.0)) }
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

    let epsilon = 0.01f32;
    let mut current = game_state.last_time;
    while (epsilon as f64) < time - current {
        Updateable::<Ball>::update(game_state, canvas_size, epsilon)?;
        Updateable::<Bat>::update(game_state, canvas_size, epsilon)?;
        Updateable::<Brick>::update(game_state, canvas_size, epsilon)?;
        current += epsilon as f64;
    }


    // game_state.ball.colliding = false;
    // game_state.collision = None;

    /*for brick in &mut game_state.bricks {
        if let Some(collision) = resolve_circle_aabb_collision(
            game_state.bat.position,
            game_state.ball.position, 
            game_state.ball.size,
            brick.position,
            brick.size * 0.5) {
            game_state.ball.colliding = true;
            game_state.collision = Some(collision);
        }
    }*/

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

    for entity in &game_state.bricks {
        entity.render(rendering_context)?;
    }
    
    game_state.ball.render(rendering_context)?;
    game_state.bat.render(rendering_context)?;

    /* if let Some(collision) = &game_state.collision {
        draw_vector(rendering_context, collision.point, collision.point + collision.normal * 32f32, "green")?;
        draw_circle(rendering_context, collision.point, 3.0, "green")?;
        draw_vector(rendering_context, collision.point, collision.point + game_state.ball.velocity, "blue")?;
        let reflected = reflect(-game_state.ball.velocity, collision.normal) * (1.0 - collision.t);
        draw_vector(rendering_context, collision.point, collision.point + reflected, "yellow")?;
    }*/

    return Ok(());
}
