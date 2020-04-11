use glm::*;
use crate::utils::*;
use crate::collision::*;
use std::mem::*;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

pub fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    v - 2f32 * dot(&v, &n) * n
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
        size : f32) -> Ball
    {
        Ball {
            position: position,
            velocity: velocity,
            size: size,
            colliding: false
        }
    }
}

pub struct Brick { pub position : Vec2, pub size : Vec2 }

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
        let origin = self.position - self.size * 0.5;
        rendering_context.set_fill_style(&JsValue::from_str("black"));
        rendering_context.fill_rect(origin.x as f64, origin.y as f64, self.size.x as f64, self.size.y as f64);
        return Ok(());
    }
}

impl Updateable<Bat> for GameState {
    fn update(
        &mut self,
        _canvas_size : Vec2,
        elapsed : f32) -> Expected<()> {

        let bat = &mut self.bat;
        bat.position += mul(bat.input * elapsed, bat.velocity);
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

        for brick in &self.bricks {
            if let Some(collision) = resolve_circle_aabb_collision(
                ball.position,
                new_position, 
                ball.size,
                brick.position,
                brick.size * 0.5) {
                outer_collision = Some(collision);
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

        // log!("before: {}, {} / {}, {}", ball.position.x, ball.position.y, ball.velocity.x, ball.velocity.y);

        if let Some(collision) = outer_collision {
            let reflected = reflect(ball.velocity, collision.normal);
            log!("normal: {}, {} / t: {}", collision.normal.x, collision.normal.y, collision.t);
            log!("normal: {}, {} / t: {}", collision.normal.x, collision.normal.y, collision.t);

            ball.position = ball.position + ball.velocity * elapsed * collision.t + reflected * elapsed* (1.0 - collision.t);
            ball.velocity = reflected;
            self.collision = outer_collision.clone();
        }
        else {
            ball.position = new_position;
        }
        

        log!("after: {}, {} / {}, {}", ball.position.x, ball.position.y, ball.velocity.x, ball.velocity.y);

        /*if ball.position.x > canvas_size.x || ball.position.x < 0.0 {
            ball.velocity.x *= -1.0;
        }

        if ball.position.y > canvas_size.y as f32 || ball.position.y < 0.0 {
            ball.velocity.y *= -1.0;
        }*/

        return Ok(());
    }
}

impl Updateable<Brick> for GameState {
    fn update(
        &mut self,
        _canvas_size : Vec2,
        _elapsed : f32) -> Expected<()> {
        return Ok(());
    }
}

pub struct GameState {
    pub bat : Bat,
    pub ball : Ball,
    pub bricks : Vec<Brick>,
    pub last_time : f64,
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
            collision: None
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
        bat_position - vec2(50.0, 0.0),
        // vec2(0.0, 0.0), 
        vec2(100.0, 0.0),
        30.0);

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
            
            let brick = Brick {
                position: bricks_origin + mul(brick_size + brick_spacing, index) + brick_origin,
                size: brick_size
            };

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

    game_state.ball.render(rendering_context)?;
    game_state.bat.render(rendering_context)?;

    for entity in &game_state.bricks {
        entity.render(rendering_context)?;
    }
    
    if let Some(collision) = &game_state.collision {
        draw_vector(rendering_context, collision.point, collision.point + collision.normal * 32f32, "green")?;
        draw_circle(rendering_context, collision.point, 3.0, "green")?;

        draw_vector(rendering_context, collision.point, collision.point + game_state.ball.velocity, "blue")?;

        let reflected = reflect(-game_state.ball.velocity, collision.normal) * (1.0 - collision.t);

        

        draw_vector(rendering_context, collision.point, collision.point + reflected, "yellow")?;

    }

    return Ok(());
}
