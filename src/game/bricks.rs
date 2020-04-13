use glm::*;
use crate::utils::*;
use crate::game::config;
use std::iter::Iterator;

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

pub struct BrickConfig {
    pub origin : Vec2,
    pub size : Vec2,
    pub width : f32,
    pub height : f32,
    pub col_width : f32,
    pub row_height : f32,
    pub spacing : Vec2
}

impl BrickConfig {
    pub fn new (canvas_size : Vec2) -> BrickConfig {
        let bricks_cols = config::NUM_BRICK_COLS;
        let brick_spacing = config::BRICK_SPACING;
        let brick_width = ((canvas_size.x - brick_spacing) / bricks_cols as f32 - brick_spacing).floor();
        let brick_height = (brick_width * 0.5f32).floor();

        return BrickConfig {
            origin: vec2(brick_width, brick_height) * 0.5f32,
            size: vec2(brick_width, brick_height),
            width: brick_width,
            height: brick_height,
            col_width: brick_width + brick_spacing,
            row_height: brick_height + brick_spacing,
            spacing: vec2(brick_spacing, brick_spacing)
        };
    }

    pub fn grid_position(&self, x : u32, y : u32) -> Vec2 {
        let index = vec2(x as f32, y as f32);
        return mul(self.size + self.spacing, index) + self.origin + self.spacing;
    }
}

pub struct Bricks {
    pub bricks : Vec<Brick>,
    pub origin : Vec2,
    pub num_cols : u32,
    pub num_rows : u32,
    pub row_shift : u32,
    pub brick_config : BrickConfig
}

impl Bricks {
    pub fn new(canvas_size : Vec2) -> Bricks {
        let mut bricks : Vec<Brick> = vec![];

        let bricks_cols = config::NUM_BRICK_COLS;
        let bricks_rows = config::NUM_BRICK_ROWS;
        let brick_config = BrickConfig::new(canvas_size);

        let brick_size = vec2(
            brick_config.width, 
            brick_config.height);

        let brick_origin = brick_size * 0.5;

        for y in 0..bricks_rows {
            for x in 0..bricks_cols {
                let index = vec2(x as f32, y as f32);
                
                let brick = Brick::new(
                    brick_config.spacing + mul(brick_size + brick_config.spacing, index) + brick_origin,
                    brick_size
                );

                bricks.push(brick);
            }
        }

        return Bricks {
            bricks: bricks,
            origin: vec2(0f32, 0f32),
            num_cols: bricks_cols,
            num_rows: bricks_rows,
            row_shift: 0,
            brick_config: brick_config
        };
    }

    pub fn last_row(&self) -> u32 {
        (self.row_shift + self.num_rows - 1) % self.num_rows
    }

    pub fn last_row_empty(&self) -> bool {
        let last_row = self.last_row();
        let begin = (last_row * self.num_cols) as usize;
        let end = begin + self.num_cols as usize;
        return self.bricks[begin..end].iter().all(|brick| brick.destruction_time.is_some());
    }

    pub fn reset_last_row(&mut self) {
        self.row_shift = self.last_row();
        self.origin += vec2(0f32, -self.brick_config.row_height);
        let begin = (self.row_shift * self.num_cols) as usize;
        let end = begin + self.num_cols as usize;

        let mut i : usize = 0;
        for brick in &mut self.bricks[begin..end] {
            brick.position = self.origin + self.brick_config.grid_position(i as u32, 0);
            brick.destruction_time = None;
            i += 1;
        }
    }

    pub fn update(
        &mut self,
        elapsed : f32) -> Expected<()> {

        let mut should_fall = false;
        let mut should_reset = false;
        let mut offset = 0f32;
           
        if self.origin.y != 0f32 {
            offset = config::BRICKS_FALLING_VELOCITY * elapsed;
            should_fall = true;

            if self.origin.y + offset >= 0f32 {
                self.origin.y = 0f32;
                should_reset = true;
            }
            else {
                self.origin.y += offset;
            }
        }

        for brick in &mut self.bricks {
            if let Some(destruction_time) = brick.destruction_time {
                brick.destruction_time = Some(destruction_time + elapsed);
            }

            if should_fall {
                if should_reset {

                }
                else {
                    brick.position.y += offset;
                }
            }
        }

        if self.last_row_empty() {
            self.reset_last_row();
        }

        return Ok(());
    }
}

