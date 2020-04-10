use glm::*;

pub struct Collision {
    pub point : Vec2,
    pub normal : Vec2,
    pub t : f32
}

pub fn circle_aabb_collides(
    circle_old_origin : Vec2,
    circle_new_origin : Vec2,
    circle_radius : f32,
    aabb_origin : Vec2,
    aabb_size : Vec2) -> Option<Collision> {
    let min_dist = aabb_size + vec2(circle_radius, circle_radius);
    let dist = abs(&(circle_new_origin - aabb_origin));
    
    if min_dist < dist {
        return None;
    }
    else {
        return None;
    }
}

