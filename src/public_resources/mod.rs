


use bevy::prelude::*;
use crate::osuparser::*;



#[derive(Component)]
pub struct OsuRing {
    pub moment_t: f32,
    pub original_scale: f32,
    pub slider_mode: bool, // time_to_shrink: f32,
}
impl OsuRing {
    pub fn new(original_scale: f32, moment_t: f32) -> Self {
        Self {
            original_scale,
            slider_mode: false,
            moment_t,
            // time_to_shrink,
        }
    }
}
// impl Default for OsuRing {
//     fn default() -> Self {
//         Self {
//             original_scale: 1.5,
//             slider_mode: false,
//             // time_to_shrink: 0.5,
//         }
//     }
// }

#[derive(Component)]
pub struct CircleInfo {
    pub clicked: bool,
    pub circle_type: OsuHitObjectType,

    pub points: Option<Vec<Vec2>>,
    pub segments: Option<usize>,
    pub slides: usize,

    pub slides_completed: usize,
    pub last_pos_idx: usize,
    pub ticks: Option<Vec<usize>>,

    pub original_pos: Vec2,

    pub moment_t: f32,

    pub size: f32,
}

impl Default for CircleInfo {
    fn default() -> Self {
        Self {
            clicked: false,
            circle_type: OsuHitObjectType::Circle(false),
            points: None,
            size: 0.0,
            last_pos_idx: 0,
            slides_completed: 0,
            ticks: None,
            moment_t: 0.0,
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

    // pub meh: Option<Mesh2d>,
    pub meh_mat: MeshMaterial2d<ColorMaterial>,

    // pub ok: Option<Mesh2d>,
    pub ok_mat: MeshMaterial2d<ColorMaterial>,

    // pub great: Option<Mesh2d>,
    pub great_mat: MeshMaterial2d<ColorMaterial>,

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

#[derive(Message)]
pub struct DrawTick {
    pub trpos: Vec2,
    pub lifetime: f32,
}


#[derive(Component)]
pub struct SliderLine {
    pub timer: Timer,
}

#[derive(Component)]
pub struct SliderTick {
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

impl BeatmapWorkerInfo {
    pub fn get_time_since_start(&self, time: f32) -> f32 {
        return time - self.started_at
    }
}



#[derive(Resource, Default)]
pub struct GeneralInfo {
    pub real_circle_radius: f32,
    pub real_hit_window: f32,
}


#[derive(Clone)]
pub enum HitScore {
    Great,
    Ok,
    Meh,
    Miss,
    SliderTickHit,
    SliderRepeatHit,
    SliderEndHit,
    ComboMiss,

}


impl HitScore {
    pub fn from_delta(delta: f32, osu_real_hit_window: &RealHitWindow) -> Self {
        if delta < osu_real_hit_window.score300 {
            Self::Great
        } else if delta < osu_real_hit_window.score100 {
            Self::Ok
        } else if delta < osu_real_hit_window.score50 {
            Self::Meh
        } else {
            Self::Miss
        }

    }

    pub fn affects_accuracy(&self) -> bool {
        match self {
            Self::Great | Self::Ok | Self::Meh | Self::Miss => {
                true
            },
            _ => {
                false
            }
        }
    }
    pub fn is_miss(&self) -> bool {
        match self {
            Self::Miss | Self::ComboMiss => {
                true
            },
            _ => {
                false
            }
        }
    }

    pub fn to_number(&self) -> i32 {
        let mut score = 0;
        match self {
            HitScore::Great => {
                score = 300;
            },
            HitScore::Ok => {
                score = 100;
            },
            HitScore::Meh => {
                score = 50;
            },
            HitScore::Miss | HitScore::ComboMiss => {
                score = 0;
            },
            HitScore::SliderTickHit => {
                score = 10;
            },
            HitScore::SliderRepeatHit | HitScore::SliderEndHit => {
                score = 30;
            }
        }
        score
    }
}


#[derive(Resource, Default)]
pub struct ScoreInfo {
    pub score: usize,
    pub accuracy: f32,
    pub current_combo: usize,
    pub hit_score: Vec<HitScore>,
}

impl ScoreInfo {
    pub fn get_accuracy(&self) -> f32 {
        let mut accuracy_sum = 0.0;

        for hit in &self.hit_score {
            match hit {
                HitScore::Great => {
                    accuracy_sum += 1.0;
                },
                HitScore::Ok => {
                    accuracy_sum += 0.66;
                },
                HitScore::Meh => {
                    accuracy_sum += 0.33;
                },
                HitScore::Miss => {
                    // no add
                },
                _ => {}
            }
        }
        accuracy_sum / (self.hit_score.len() as f32)
    }
}






#[derive(Message)]
pub struct AddScore {
    pub score: HitScore,
}


//Score = ((700000 * combo_bonus / max_combo_bonus) + (300000 * ((accuracy_percentage / 100) ^ 10) * elapsed_objects / total_objects) + spinner_bonus) * mod_multiplier
impl AddScore {
    
    pub fn new(score: HitScore) -> Self {
        Self {score}
    }
}



#[derive(Component)]
pub struct ScoreGui;

#[derive(Component)]
pub struct AccuracyGui;

#[derive(Component)]
pub struct ComboGui;


pub enum TickType {
    SliderTick,
    SliderEnd,
    SliderRepeat
}
impl TickType {
    pub fn to_hit_score(&self) -> HitScore {
        match self {
            Self::SliderTick => {
                HitScore::SliderTickHit
            },
            Self::SliderRepeat => {
                HitScore::SliderRepeatHit
            },
            Self::SliderEnd => {
                HitScore::SliderEndHit
            }
        }
    }
}



#[derive(Message)]
pub struct TickCheck {
    pub trpos: Vec2,
    pub tick_type: TickType,
}

