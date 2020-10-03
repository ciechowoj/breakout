use glm::*;

fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
}

pub fn fmin(a: f32, b: f32) -> f32 { if a < b { a } else { b } }
pub fn fmax(a: f32, b: f32) -> f32 { if a < b { b } else { a } }

pub struct Bat {
    pub position : Vec2,
    pub velocity : Vec2,
    pub size : Vec2,
    pub input : Vec2
}

impl Bat {
    pub fn new(canvas_size : Vec2) -> Bat {
        let bat_position = vec2(canvas_size.x * 0.5, canvas_size.y - 100.0);

        Bat {
            position: bat_position,
            velocity: vec2(1000.0, 1000.0),
            size: vec2(200.0, 20.0),
            input: vec2(0.0, 0.0)
        }
    }
}

pub fn update_bat(
    bat : &mut Bat,
    canvas_size : Vec2,
    elapsed : f32) -> anyhow::Result<()> {

    bat.position += mul(bat.input * elapsed, bat.velocity);

    bat.position.x -= fmin(bat.position.x - bat.size.x * 0.5, 0f32);
    bat.position.x -= fmax(bat.position.x + bat.size.x * 0.5 - canvas_size.x, 0f32);

    return Ok(());
}
