use bevy::prelude::*;


#[derive(Clone, Copy, Debug)]
pub struct CubicSegment2d {
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
}

impl CubicSegment2d {
    fn sample(&self, u: f32) -> Vec2 {
        (((self.a * u) + self.b) * u + self.c) * u + self.d
    }

    fn tangent(&self, u: f32) -> Vec2 {
        (3.0 * self.a * u + 2.0 * self.b) * u + self.c
    }

    fn length(&self, steps: usize) -> f32 {
        let mut total = 0.0;
        let mut prev = self.sample(0.0);

        for i in 1..=steps {
            let u = i as f32 / steps as f32;
            let p = self.sample(u);
            total += prev.distance(p);
            prev = p;
        }

        total
    }
}

fn safe_dt(a: Vec2, b: Vec2) -> f32 {
    let d = a.distance(b).sqrt(); // alpha = 0.5 => centripetal
    d.max(1e-4)
}

fn centripetal_catmull_rom_segment(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
) -> CubicSegment2d {
    // Non-uniform knot spacing for centripetal Catmull–Rom
    let t0 = 0.0;
    let t1 = t0 + safe_dt(p0, p1);
    let t2 = t1 + safe_dt(p1, p2);
    let t3 = t2 + safe_dt(p2, p3);

    // Tangent at p1 and p2 for the segment p1 -> p2
    // These are derivatives with respect to the knot parameter.
    let m1 =
        (p2 - p1) / (t2 - t1)
        + (p1 - p0) / (t1 - t0)
        - (p2 - p0) / (t2 - t0);

    let m1 = m1 * (t2 - t1);

    let m2 =
        (p3 - p2) / (t3 - t2)
        + (p2 - p1) / (t2 - t1)
        - (p3 - p1) / (t3 - t1);

    let m2 = m2 * (t2 - t1);

    // Hermite -> power basis
    let a = 2.0 * p1 - 2.0 * p2 + m1 + m2;
    let b = -3.0 * p1 + 3.0 * p2 - 2.0 * m1 - m2;
    let c = m1;
    let d = p1;

    CubicSegment2d { a, b, c, d }
}


#[derive(Resource, Debug)]
pub struct CentripetalCatmullRomSpline2d {
    pub segments: Vec<CubicSegment2d>,
}

impl CentripetalCatmullRomSpline2d {
    pub fn new(points: &[Vec2]) -> Self {
        assert!(points.len() >= 4, "Need at least 4 points");

        let mut segments = Vec::with_capacity(points.len().saturating_sub(3));

        for i in 0..points.len() - 3 {
            segments.push(centripetal_catmull_rom_segment(
                points[i],
                points[i + 1],
                points[i + 2],
                points[i + 3],
            ));
        }

        Self { segments }
    }

    pub fn sample(&self, t: f32) -> Vec2 {
        let max_t = self.segments.len() as f32;
        let t = t.clamp(0.0, max_t);

        let mut seg_idx = t.floor() as usize;
        let mut u = t - seg_idx as f32;

        if seg_idx >= self.segments.len() {
            seg_idx = self.segments.len() - 1;
            u = 1.0;
        }

        self.segments[seg_idx].sample(u)
    }

    pub fn tangent(&self, t: f32) -> Vec2 {
        let max_t = self.segments.len() as f32;
        let t = t.clamp(0.0, max_t);

        let mut seg_idx = t.floor() as usize;
        let mut u = t - seg_idx as f32;

        if seg_idx >= self.segments.len() {
            seg_idx = self.segments.len() - 1;
            u = 1.0;
        }

        self.segments[seg_idx].tangent(u)
    }

    pub fn length(&self, steps_per_segment: usize) -> f32 {
        self.segments
            .iter()
            .map(|seg| seg.length(steps_per_segment))
            .sum()
    }
}