use glm::*;
use crate::game::config;

fn mul(a: Vec2, b: Vec2) -> Vec2 {
    vec2(a.x * b.x, a.y * b.y)
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

pub struct Bricks {
    pub bricks : Vec<Brick>,
    pub origin : Vec2,
    pub num_bricks_rows : usize,
    pub top_row_index : usize
}

impl Bricks {
    pub fn new(canvas_size : Vec2) -> Bricks {
        let mut bricks : Vec<Brick> = vec![];

        let bricks_cols = config::NUM_BRICK_COLS;
        let bricks_rows = config::NUM_BRICK_ROWS;
        let brick_spacing = config::BRICK_SPACING;

        let brick_width = (canvas_size.x - brick_spacing) / bricks_cols as f32 - brick_spacing;
        let brick_height = brick_width * 0.5f32;

        let brick_size = vec2(
            brick_width.floor(), 
            brick_height.floor());

        let bricks_origin = vec2(brick_spacing, brick_spacing);
        let brick_origin = brick_size * 0.5;

        for y in 0..bricks_rows {
            for x in 0..bricks_cols {
                let index = vec2(x as f32, y as f32);
                
                let brick = Brick::new(
                    bricks_origin + mul(brick_size + vec2(brick_spacing, brick_spacing), index) + brick_origin,
                    brick_size
                );

                bricks.push(brick);
            }
        }

        return Bricks {
            bricks: bricks,
            origin: brick_origin,
            num_bricks_rows: 0,
            top_row_index: 0
        };
    }

    fn update(
        &mut self,
        _canvas_size : Vec2,
        elapsed : f32) -> Expected<()> {

        for brick in &mut self.bricks.bricks {
            if let Some(destruction_time) = brick.destruction_time {
                brick.destruction_time = Some(destruction_time + elapsed);
            }
        }

        return Ok(());
    }
}

