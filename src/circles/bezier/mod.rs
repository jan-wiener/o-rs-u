use bevy::prelude::*;


pub fn bezier_casteljau(points: &[Vec2], t: f32) -> Vec2 {
    let mut pts = points.to_vec();
    let n = pts.len();
    for k in 1..n {
        for i in 0..(n - k) {
            pts[i] = pts[i].lerp(pts[i + 1], t);
        }
    }
    pts[0]
}

pub fn bezier_length(points: &[Vec2], steps: usize) -> f32 {
    let mut total = 0.0;
    let mut prev = bezier_casteljau(points, 0.0);

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let p = bezier_casteljau(points, t);
        total += prev.distance(p);
        prev = p;
    }

    total
}


