use glm::*;
use crate::utils::*;

pub struct Collision {
    pub point : Vec2,
    pub normal : Vec2,
    pub t : f32
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
        aabb_radius : Vec2) -> Option<Vec2> {
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
                        return Some(v_lt);
                    }
                }
                else if circle_origin.y < bottom {
                    return Some(vec2(left, circle_origin.y));
                }
                else {
                    if radius2 < distance2(&circle_origin, &v_lb) {
                        return None;
                    }
                    else {
                        return Some(v_lb);
                    }
                }
            }
            else if circle_origin.x < right {
                if circle_origin.y < top {
                    return Some(vec2(circle_origin.x, top));
                }
                else if circle_origin.y < bottom {
                    return Some(circle_origin);
                }
                else {
                    return Some(vec2(circle_origin.x, bottom));
                }
            }
            else {
                if circle_origin.y < top {
                    if radius2 < distance2(&circle_origin, &v_rt) {
                        return None;
                    }
                    else {
                        return Some(v_rt);
                    }
                }
                else if circle_origin.y < bottom {
                    return Some(vec2(right, circle_origin.y));
                }
                else {
                    if radius2 < distance2(&circle_origin, &v_rb) {
                        return None;
                    }
                    else {
                        return Some(v_rb);
                    }
                }
            }
        }
        else {
            return None;
        }
    }
    
    if let Some(collision_point) = circle_aabb_collision(circle_new_origin, circle_radius, aabb_origin, aabb_radius) {
        let mut a = 0f32;
        let mut b = 1f32;

        let mut collision_point = collision_point;
        let mut circle_origin = circle_new_origin;
        let mut t = a;

        while 0.00001f32 < (a - b).abs() {
            t = a + (b - a) * 0.5;
            circle_origin = mix(&circle_old_origin, &circle_new_origin, t);
            let new_collision_point = circle_aabb_collision(circle_origin, circle_radius, aabb_origin, aabb_radius);

            if let Some(point) = new_collision_point {
                collision_point = point;
                b = t;
            }
            else {
                a = t;
            }
        }

        return Some(Collision {
            point: collision_point,
            normal: normalize(&(circle_origin - collision_point)),
            t: t
        });
    }

    return None;
}
