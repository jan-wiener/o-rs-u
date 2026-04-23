use bevy::prelude::*;




pub fn vec_vec2_len(vec2s: &[Vec2]) -> f32 {
    let mut total = 0.0;
    let _last: &Vec2;
    for i in 1..vec2s.len() {
        total += vec2s[i - 1].distance(vec2s[i]);
    }
    total
}




pub fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

pub fn line_intersection(p: Vec2, r: Vec2, q: Vec2, s: Vec2) -> Option<Vec2> {
    let rxs = cross(r, s);
    if rxs.abs() < 1e-6 {
        return None; // parallel
    }

    let t = cross(q - p, s) / rxs;
    Some(p + r * t)
}

pub fn norm_angle(a: f32) -> f32 {
    if a < 0.0 {
        a + std::f32::consts::TAU
    } else {
        a
    }
}

pub fn ccw_diff(a: f32, b: f32) -> f32 {
    let mut d = b - a;
    if d < 0.0 {
        d += std::f32::consts::TAU;
    }
    d
}


