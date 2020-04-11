use glm::*;
use crate::utils::*;

#[derive(Copy, Clone)]
pub struct Collision {
    pub point : Vec2,
    pub normal : Vec2,
    pub t : f32
}

const COLLISION_EPSILON : f32 = 0.000001f32;

fn search_exact_collision_point<F>(
    collision_point : (Vec2, Vec2),
    old_origin : Vec2,
    new_origin : Vec2,
    test_collision : &mut F) -> Collision
    where F : FnMut(Vec2) -> Option<(Vec2, Vec2)> {

    let mut a = 0f32;
    let mut b = 1f32;

    let mut collision_point = collision_point;
    let mut circle_origin = new_origin;

    while COLLISION_EPSILON < (a - b).abs() {
        let t = a + (b - a) * 0.5;
        circle_origin = mix(&old_origin, &new_origin, t);
        let new_collision_point = test_collision(circle_origin);

        if let Some(point) = new_collision_point {
            collision_point = point;
            b = t;
        }
        else {
            a = t;
        }
    }

    return Collision {
        point: collision_point.0,
        normal: collision_point.1,
        t: a
    };
}

pub fn resolve_circle_aabb_collision(
    circle_old_origin : Vec2,
    circle_new_origin : Vec2,
    circle_radius : f32,
    aabb_origin : Vec2,
    aabb_radius : Vec2) -> Option<Collision> {
    fn circle_aabb_collision(
        circle_origin : Vec2,
        circle_radius : f32,
        aabb_origin : Vec2,
        aabb_radius : Vec2) -> Option<(Vec2, Vec2)> {
        let min_dist = aabb_radius + vec2(circle_radius, circle_radius);
        let dist = abs(&(circle_origin - aabb_origin));
        
        if dist <= min_dist {
            let radius2 = circle_radius * circle_radius;
            let left = aabb_origin.x - aabb_radius.x;
            let right = aabb_origin.x + aabb_radius.x;
            let top = aabb_origin.y - aabb_radius.y;
            let bottom = aabb_origin.y + aabb_radius.y;
            let v_lt = vec2(left, top);
            let v_rt = vec2(right, top);
            let v_lb = vec2(left, bottom);
            let v_rb = vec2(right, bottom);
    
            if circle_origin.x < left {
                if circle_origin.y < top {
                    if radius2 < distance2(&circle_origin, &v_lt) {
                        return None;
                    }
                    else {
                        let normal = normalize(&(circle_origin - v_lt));
                        return Some((v_lt, normal));
                    }
                }
                else if circle_origin.y < bottom {
                    let normal = vec2(-1f32, 0f32);
                    return Some((vec2(left, circle_origin.y), normal));
                }
                else {
                    if radius2 < distance2(&circle_origin, &v_lb) {
                        return None;
                    }
                    else {
                        let normal = normalize(&(circle_origin - v_lb));
                        return Some((v_lb, normal));
                    }
                }
            }
            else if circle_origin.x < right {
                if circle_origin.y < top {
                    let normal = vec2(0f32, -1f32);
                    return Some((vec2(circle_origin.x, top), normal));
                }
                else if circle_origin.y < bottom {
                    let normal = vec2(-1f32, 0f32); // FIXME
                    return Some((circle_origin, normal));
                }
                else {
                    let normal = vec2(0f32, 1f32);
                    return Some((vec2(circle_origin.x, bottom), normal));
                }
            }
            else {
                if circle_origin.y < top {
                    if radius2 < distance2(&circle_origin, &v_rt) {
                        return None;
                    }
                    else {
                        let normal = normalize(&(circle_origin - v_rt));
                        return Some((v_rt, normal));
                    }
                }
                else if circle_origin.y < bottom {
                    let normal = vec2(1f32, 0f32);
                    return Some((vec2(right, circle_origin.y), normal));
                }
                else {
                    if radius2 < distance2(&circle_origin, &v_rb) {
                        return None;
                    }
                    else {
                        let normal = normalize(&(circle_origin - v_rb));
                        return Some((v_rb, normal));
                    }
                }
            }
        }
        else {
            return None;
        }
    }

    let mut test_collision = |origin| {
        circle_aabb_collision(origin, circle_radius, aabb_origin, aabb_radius)
    };
    
    if let Some(collision_point) = test_collision(circle_new_origin) {
        return Some(search_exact_collision_point(
            collision_point,
            circle_old_origin,
            circle_new_origin,
            &mut test_collision));
    }

    return None;
}

pub fn resolve_circle_aabb_inv_collision(
    circle_old_origin : Vec2,
    circle_new_origin : Vec2,
    circle_radius : f32,
    aabb_origin : Vec2,
    aabb_radius : Vec2) -> Option<Collision> {
    fn circle_aabb_collision(
        circle_origin : Vec2,
        circle_radius : f32,
        aabb_origin : Vec2,
        aabb_radius : Vec2) -> Option<(Vec2, Vec2)> {
        let max_dist = aabb_radius - vec2(circle_radius, circle_radius);
        let dist = abs(&(circle_origin - aabb_origin));

        if dist.x >= max_dist.x || dist.y >= max_dist.y {
            let left = aabb_origin.x - aabb_radius.x;
            let right = aabb_origin.x + aabb_radius.x;
            let top = aabb_origin.y - aabb_radius.y;
            let bottom = aabb_origin.y + aabb_radius.y;
            let left_dist = (circle_origin.x - left).abs();
            let right_dist = (circle_origin.x - right).abs();
            let top_dist = (circle_origin.y - top).abs();
            let bottom_dist = (circle_origin.y - bottom).abs();

            let mut min_dist = left_dist;
            let mut collision_point = vec2(left, circle_origin.y);
            let mut normal = vec2(1f32, 0f32);

            if right_dist < min_dist { 
                collision_point = vec2(right, circle_origin.y);
                normal = vec2(-1f32, 0f32);
                min_dist = right_dist;
            }
            
            if top_dist < min_dist { 
                collision_point = vec2(circle_origin.x, top);
                normal = vec2(0f32, 1f32);
                min_dist = top_dist;
            }

            if bottom_dist < min_dist {
                collision_point = vec2(circle_origin.x, bottom);
                normal = vec2(0f32, -1f32);
            }

            return Some((collision_point, normal));
        }
        else {
            return None;
        }
    }

    let mut test_collision = |origin| {
        circle_aabb_collision(origin, circle_radius, aabb_origin, aabb_radius)
    };
    
    if let Some(collision_point) = test_collision(circle_new_origin) {
        return Some(search_exact_collision_point(
            collision_point,
            circle_old_origin,
            circle_new_origin,
            &mut test_collision));
    }

    return None;
}