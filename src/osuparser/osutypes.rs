use bevy::prelude::*;


// Osu Hit Object
#[allow(dead_code)] // inner bools should be combo start identifiers. idk if I will implement it tho.
#[derive(Debug, Clone)]
pub enum OsuHitObjectType {
    
    Circle(bool),
    Slider(bool),
    Spinner(bool),
    // Tick,
    // SliderEnd,
}

impl Default for OsuHitObjectType {
    fn default() -> Self {
        Self::Circle(false)
    }
}

impl OsuHitObjectType {
    pub fn is_circle_like(&self) -> bool {
        match self {
            Self::Circle(_) | Self::Slider(_) => true,
            _ => false,
        }
    }
}


// --------


#[derive(Debug, Default, Clone)]
pub enum OsuHitSound {
    #[default]
    Normal,
    Whistle,
    Finish,
    Clap,
}


// -------

#[derive(Debug, Clone)]
pub enum CurveType {
    Bezier,
    Centripetal,
    Linear,
    PerfectCircle,
}


// -------

#[derive(Debug, Default, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}




impl Point {
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn to_real_pos(&self, screen_size: Vec2) -> Vec2 {
        // let osu_to_window_ratio = (screen_size.y*0.8) / 384.0;
        let osu_to_window_ratio = screen_size.y / 480.0;
        // let osu_to_window_ratio = 1.0;

        let original: Vec2 = self.to_vec2();

        let original_from_middle = Vec2::new(original.x - 256.0, 192.0 - original.y);
        let translated = original_from_middle * osu_to_window_ratio;

        translated
    }
}
impl Into<Vec2> for Point {
    fn into(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}


// -------

#[derive(Debug, Clone)]
pub struct Slider {
    pub curve_type: CurveType,
    pub curve_points: Vec<Point>,
    pub trcurve_points: Vec<Vec2>,
    pub slides: usize,
    // pub length: f64,
}

// -------


#[derive(Debug, Clone)]
pub struct Spinner {}

// -------


#[derive(Debug, Default, Message, Clone)]
pub struct OsuHitObject {
    pub pos: Point,
    pub trpos: Option<Vec2>,
    pub time: f32,
    pub hitobjecttype: OsuHitObjectType,
    pub hitsound: OsuHitSound,
    pub slider_params: Option<Slider>,
    pub spinner_params: Option<Spinner>,

    pub ticks: Option<Vec<usize>>,
    pub points: Option<Vec<Vec2>>,
    pub segments: Option<usize>,
    pub slides: usize,
    pub length: f32,
}


// -------


#[derive(Debug)]
pub struct OsuDifficulty {
    pub hp_drain: f32,
    pub circle_size: f32,
    pub overall_diff: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,
    pub slider_tick_rate: f32,
}

impl Default for OsuDifficulty {
    fn default() -> Self {
        Self {
            hp_drain: 5.0,
            circle_size: 5.0,
            approach_rate: 8.0,
            overall_diff: 5.0,
            slider_multiplier: 5.0,
            slider_tick_rate: 1.0,
        }
    }
}

// -------


#[derive(Default)]
pub struct RealHitWindow {
    pub score300: f32,
    pub score100: f32,
    pub score50: f32,
}


// -------


#[derive(Default, Debug, Clone)]
pub struct OsuTimingPoint {
    pub time: i32,
    pub beat_length: f32,
    pub meter: i32,
    pub sample_set: i32,
    pub sample_index: i32,
    pub volume: i32,
    pub uninherited: bool,
    pub effects: i32,
}
impl OsuTimingPoint {
    fn new(
        time: i32,
        beat_length: f32,
        meter: i32,
        sample_set: i32,
        sample_index: i32,
        volume: i32,
        uninherited: bool,
        effects: i32,
    ) -> Self {
        Self {
            time,
            beat_length,
            meter,
            sample_set,
            sample_index,
            volume,
            uninherited,
            effects,
        }
    }
    pub fn from_tuple(t: (i32, f32, i32, i32, i32, i32, bool, i32)) -> Self {
        OsuTimingPoint::new(t.0, t.1, t.2, t.3, t.4, t.5, t.6, t.7)
    }
}


// -------

#[derive(Default)]
pub struct OsuBackgroundEvent {
    pub filename: String,
    pub offset: Vec2,
    pub real_offset: Vec2,
}

// -------


