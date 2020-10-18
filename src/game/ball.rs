use glm::*;
use crate::game::config;
use crate::game::bat::*;
use crate::game::bricks::*;
use crate::utils::*;
use crate::collision::*;
use js_sys::Math::random;

fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

pub fn reflect(v: Vec2, n: Vec2) -> Vec2 {
    let v_dot_n = dot(&v, &n);
    if v_dot_n < 0.0 { v - 2f32 * v_dot_n * n } else { v }
}

pub struct Ball {
    pub position : Vec2,
    pub velocity : Vec2,
    pub size : f32,
    pub colliding : bool,
    pub freeze_time : Option<f32>
}

impl Ball {
    pub fn new() -> Ball {
        let mut ball = Ball {
            position: vec2(0f32, 0f32),
            velocity: vec2(0f32, 0f32),
            size: config::BALL_START_SIZE,
            colliding: false,
            freeze_time: None
        };

        ball.reset_position();

        return ball;
    }

    pub fn reset_position(&mut self) {
        let ball_start_position = vec2(config::BALL_START_X, config::BALL_START_Y);
        let ball_start_angle = config::BALL_START_ANGLE * (random() as f32 - 0.5f32);
        let ball_start_direction = rotate_vec2(&vec2(0f32, -1f32), ball_start_angle);

        self.position = mul(ball_start_position, vec2(726f32, 968f32));
        self.velocity = ball_start_direction * config::BALL_VELOCITY;

        self.freeze_time = Some(0f32);
    }

    pub fn effective_velocity(&self) -> Vec2 {
        return self.velocity * match self.freeze_time { Some(_) => 0f32, None => 1f32 };
    }
}

pub struct BallStatus {
    pub brick_hit_count : u32,
    pub out_of_arena : bool
}

impl BallStatus {
    fn new() -> BallStatus {
        BallStatus { brick_hit_count : 0, out_of_arena : false }
    }
}

pub fn update_ball(
    bat : &Bat,
    ball : &mut Ball,
    bricks : &mut Bricks,
    elapsed : f32) -> anyhow::Result<BallStatus> {

    let new_position = ball.position + ball.effective_velocity() * elapsed;
    let mut outer_collision : Option<Collision> = None;

    let mut result = BallStatus::new();

    for brick in &mut bricks.bricks {
        if let None = brick.destruction_time {
            if let Some(collision) = resolve_circle_aabb_collision(
                ball.position,
                new_position,
                ball.size,
                brick.position,
                brick.size * 0.5) {
                outer_collision = Some(collision);
                brick.destruction_time = Some(0f32);
                result.brick_hit_count += 1;
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

    let game_area = vec2(config::GAME_AREA_WIDTH as f32, config::GAME_AREA_HEIGHT as f32);

    if let Some(collision) = resolve_circle_aabb_inv_collision(
        ball.position,
        new_position,
        ball.size,
        game_area * 0.5,
        game_area * 0.5) {
        if collision.normal != vec2(0f32, -1f32) {
            outer_collision = Some(collision);
        }
    }

    if let Some(collision) = outer_collision {
        let reflected = reflect(ball.velocity, collision.normal);
        ball.position = ball.position + ball.velocity * elapsed * collision.t + reflected * elapsed * (1.0 - collision.t);
        ball.velocity = reflected;
    }
    else {
        ball.position = new_position;
    }

    ball.freeze_time = match ball.freeze_time {
        Some(time) => if time > config::BALL_FREEZE_TIME { None } else { Some(time + elapsed) },
        None => None
    };

    if ball.position.y - ball.size > game_area.y {
        result.out_of_arena = true;
    }

    return Ok(result);
}

pub fn draw_circle(
    rendering_context : &web_sys::CanvasRenderingContext2d,
    origin : Vec2,
    radius : f32,
    color : &'static str) -> anyhow::Result<()> {
    rendering_context.begin_path();
    rendering_context.arc(origin.x as f64, origin.y as f64, radius as f64, 0.0, two_pi()).to_anyhow()?;
    rendering_context.set_fill_style(&wasm_bindgen::JsValue::from_str(color));
    rendering_context.fill();
    return Ok(());
}

pub fn render_ball(ball : &Ball, rendering_context : &web_sys::CanvasRenderingContext2d) -> anyhow::Result<()> {
    let color = match ball.freeze_time {
        Some(_) => "grey",
        None => if ball.colliding { "red" } else { "black" }
    };

    draw_circle(rendering_context, ball.position, ball.size, color)?;
    return Ok(());
}