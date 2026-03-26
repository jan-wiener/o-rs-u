


use bevy::prelude::*;
use crate::osuparser::*;



#[derive(Component)]
pub struct OsuRing {
    pub original_scale: f32,
    pub slider_mode: bool, // time_to_shrink: f32,
}
impl OsuRing {
    pub fn new(original_scale: f32) -> Self {
        Self {
            original_scale,
            slider_mode: false,
            // time_to_shrink,
        }
    }
}
impl Default for OsuRing {
    fn default() -> Self {
        Self {
            original_scale: 1.5,
            slider_mode: false,
            // time_to_shrink: 0.5,
        }
    }
}

#[derive(Component)]
pub struct CircleInfo {
    pub clicked: bool,
    pub circle_type: OsuHitObjectType,

    pub points: Option<Vec<Vec2>>,
    pub segments: Option<usize>,
    pub slides: u32,

    pub original_pos: Vec2,

    pub size: f32,
}

impl Default for CircleInfo {
    fn default() -> Self {
        Self {
            clicked: false,
            circle_type: OsuHitObjectType::Circle(false),
            points: None,
            size: 0.0,
            segments: None,
            original_pos: Vec2::default(),
            slides: 1,
        }
    }
}



#[derive(Message)]
pub struct RemoveCircle {
    pub entity: Entity,
}


#[derive(Resource)]
pub struct CircleMaterials {
    pub main: Mesh2d,
    pub main_mat: MeshMaterial2d<ColorMaterial>,
    pub ring: Mesh2d,
    pub ring_mat: MeshMaterial2d<ColorMaterial>,
}



#[derive(Message)]
pub struct StartMovingSlider {
    pub entity: Entity,
}

pub struct MovingSlider {
    pub entity: Entity,
    pub started_at: f32,
    pub done_slides: u32,
    // target_slides: u32
}
#[derive(Resource, Default)]
pub struct MovingSlidersRes {
    pub sliders: Vec<MovingSlider>,
}


#[derive(Resource, Default)]
pub struct MouseInfo {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub on_screen: bool,
    pub pressed: bool,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LineGizmos {}

#[derive(Message)]
pub struct DrawLine {
    pub points: Vec<Vec2>,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct SliderLine {
    pub timer: Timer,
}


#[derive(Message)]
pub struct LoadBeatmap {
    pub path: String,
}

#[derive(Resource, Default)]
pub struct BeatmapWorkerInfo {
    pub start: bool,
    pub started_at: f32,
    pub next: usize,
}

#[derive(Resource, Default)]
pub struct GeneralInfo {
    pub real_circle_radius: f32,
}



pub enum HitScore {
    Great,
    Ok,
    Meh,
    Miss,
}


#[derive(Resource, Default)]
pub struct ScoreInfo {
    pub score: usize,
    pub accuracy: f32,
    pub hit_score: Vec<HitScore>,
}


