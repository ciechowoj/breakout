use std::ops::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct vec2 {
    pub x : f32,
    pub y : f32
}

impl Mul<f32> for vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        vec2 { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<f32> for &vec2 {
    type Output = vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        vec2 { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Mul<vec2> for f32 {
    type Output = vec2;

    fn mul(self, rhs: vec2) -> Self::Output {
        vec2 { x: self * rhs.x, y: self * rhs.y }
    }
}

impl Add for vec2 {
    type Output = Self;

    fn add(self, rhs: vec2) -> Self::Output {
        vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for vec2 {
    type Output = Self;

    fn sub(self, rhs: vec2) -> Self::Output {
        vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Sub for &vec2 {
    type Output = vec2;

    fn sub(self, rhs: &vec2) -> Self::Output {
        vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Sub<&vec2> for &vec2 {
    type Output = vec2;

    fn sub(self, rhs: &vec2) -> Self::Output {
        vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}