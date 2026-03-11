use std::{error::Error, fs, io::Read, path::Path};

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub enum OsuHitObjectType {
    Circle(bool),
    Slider(bool),
    Spinner(bool),
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

#[derive(Debug, Default, Clone)]
enum OsuHitSound {
    #[default]
    Normal,
    Whistle,
    Finish,
    Clap,
}

#[derive(Debug, Clone)]
pub enum CurveType {
    Bezier,
    Centripetal,
    Linear,
    PerfectCircle,
}

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

#[derive(Debug, Clone)]
pub struct Slider {
    pub curve_type: CurveType,
    pub curve_points: Vec<Point>,
    pub trcurve_points: Vec<Vec2>,
    pub slides: u32,
    pub length: f64,
}

#[derive(Debug, Clone)]
struct Spinner {}

#[derive(Debug, Default, Message, Clone)]
pub struct OsuHitObject {
    pub pos: Point,
    pub trpos: Option<Vec2>,
    pub time: f32,
    pub hitobjecttype: OsuHitObjectType,
    pub hitsound: OsuHitSound,
    pub slider_params: Option<Slider>,
    pub spinner_params: Option<Spinner>,
}


#[derive(Debug)]
pub struct OsuDifficulty {
    pub hp_drain: f32,
    pub circle_size: f32,
    pub overall_diff: f32,
    pub approach_rate: f32,
    pub slider_multiplier: f32,


}

impl Default for OsuDifficulty {
    fn default() -> Self {
        Self {
            hp_drain: 5.0,
            circle_size: 5.0,
            approach_rate: 8.0,
            overall_diff: 5.0,
            slider_multiplier: 5.0
        }
    }
}

#[derive(Default)]
struct RealHitWindow {
    pub score300: f32,
    pub score100: f32,
    pub score50: f32
}


#[derive(Default)]
struct OsuTimingPoint {
    time: i32,
    beat_length: f32,
    meter: i32,
    sample_set: i32,
    sample_index: i32,
    volume: i32,
    uninherited: bool,
    effects: i32
}


#[derive(Resource, Default)]
pub struct OsuBeatmap {
    pub hit_objects: Vec<OsuHitObject>,
    pub difficulty: OsuDifficulty,

    
    pub timing_points: Vec<OsuTimingPoint>,

    pub real_circle_size: f32,
    pub real_hit_window: RealHitWindow,
    pub real_approach_time: f32,


    pub screen_size: Vec2,
}

impl OsuBeatmap {   
    pub fn calc_real_values(&mut self) {
        let score300 = 80.0 - 6.0 * self.difficulty.overall_diff;
        let score100 = 140.0 - 8.0 * self.difficulty.overall_diff;
        let score50 = 200.0 - 10.0 * self.difficulty.overall_diff;

        self.real_hit_window = RealHitWindow { 
            score300,
            score100,
            score50
        };

        self.real_circle_size = (self.screen_size.y / 480.0 ) * (54.4 - 4.48 * (self.difficulty.circle_size as f32));

        if self.difficulty.approach_rate < 5.0 {
            self.real_approach_time = 1200.0 + 120.0 * (5.0 - self.difficulty.approach_rate);
        } else {
            self.real_approach_time = 1200.0 - 150.0 * (self.difficulty.approach_rate - 5.0);
        }
        self.real_approach_time /= 1000.0;

    }

    pub fn get_time_to_complete_slider(&self, length: f32) -> f32 {
        return (((length * (480.0/self.screen_size.y)) / (self.difficulty.slider_multiplier * 100.0 * 1.0)) * 434.0) / 1000.0; 
    }

    pub fn get_real_circle_size(&self) -> f32 {
        let x = 54.4 - 4.48 * (self.difficulty.circle_size as f32);
        (self.screen_size.y / 480.0) * x
    }
}

fn str_to_line_vec(s: &str) -> Vec<String> {
    s.trim().to_string()
        .split("\n")
        .map(|x| return x.trim().to_string())
        .collect::<Vec<String>>()
}

//std::io::Result<OsuBeatmap>
pub fn parse_osu_file(p: &Path) -> std::io::Result<OsuBeatmap> {
    let mut f = fs::File::open(p)?;
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let mut split_s = s
        .split("]\r")
        .map(|x| return x.to_string())
        .collect::<Vec<String>>();
 
    let mut general = str_to_line_vec(split_s[1].as_str());
    let mut editor = str_to_line_vec(split_s[2].as_str());
    let mut metadata = str_to_line_vec(split_s[3].as_str());
    let mut difficulty = str_to_line_vec(split_s[4].as_str());
    let mut events = str_to_line_vec(split_s[5].as_str());
    let mut timing_points = str_to_line_vec(split_s[6].as_str());
    let mut colors = str_to_line_vec(split_s[7].as_str());
    difficulty.pop();
    difficulty.pop();

   

    let beatmap_str = split_s[8].trim();
    let beatmap_vec = str_to_line_vec(split_s[8].as_str());

    let mut osu_beatmap = OsuBeatmap::default();


    let difficulty_split = difficulty.iter().map(|org| {
        let items = org.split(":").map(|x| x.to_string()).collect::<Vec<String>>();
        (items[0].to_owned(), items[1].parse::<f32>().unwrap())
    }).collect::<Vec<(String, f32)>>();

    for (name, value) in difficulty_split {
        match name.as_str() {
            "HPDrainRate" => osu_beatmap.difficulty.hp_drain = value, 
            "CircleSize" => osu_beatmap.difficulty.circle_size = value, 
            "OverallDifficulty" => osu_beatmap.difficulty.overall_diff = value, 
            "ApproachRate" => osu_beatmap.difficulty.approach_rate = value, 
            "SliderMultiplier" => osu_beatmap.difficulty.slider_multiplier = value, 
            _=>{}
        }
    }
    println!("diffs: {:?}", osu_beatmap.difficulty);

    // osu_beatmap.difficulty = difficulty_split[2].1.parse().unwrap();




    // println!("{:?}", split_s);

    for beat in beatmap_vec {
        let beat_info = beat
            .split(",")
            .map(|x| return x.to_string())
            .collect::<Vec<String>>();

        let type_u: u8 = beat_info[3].parse().unwrap();
        let type_bits = (0..8).map(|i| (type_u >> i) & 1).collect::<Vec<u8>>();

        let hitobjecttype: OsuHitObjectType;
        let combostart: bool;
        if type_bits[2] == 1 {
            combostart = true;
        } else {
            combostart = false;
        }
        if type_bits[0] == 1 {
            hitobjecttype = OsuHitObjectType::Circle(combostart);
        } else if type_bits[1] == 1 {
            hitobjecttype = OsuHitObjectType::Slider(combostart);
        } else if type_bits[3] == 1 {
            hitobjecttype = OsuHitObjectType::Spinner(combostart);
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Hit object not registered",
            ));
        }

        let x: i32 = beat_info[0].parse().unwrap();
        let y: i32 = beat_info[1].parse().unwrap();
        let t: f32 = beat_info[2].parse::<f32>().unwrap() / 1000.0;

        // println!("{:?}", beat_info);

        let mut slider: Option<Slider> = None;
        if let OsuHitObjectType::Slider(_) = hitobjecttype {
            // let sliderinfo = &beat_info[5..=7];
            let curvesinfo = &beat_info[5];
            let mut curve_vec = curvesinfo
                .split("|")
                .map(|x| return x.to_string())
                .collect::<Vec<String>>();

            let curve_type_s = curve_vec.remove(0);

            println!("CurveType: {}", curve_type_s);

            let curve_type = match curve_type_s.as_str() {
                "B" => CurveType::Bezier,
                "C" => CurveType::Centripetal,
                "L" => CurveType::Linear,
                "P" => CurveType::PerfectCircle,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Invalid crve type",
                    ));
                }
            };

            let curve_points: Vec<Point> = curve_vec
                .into_iter()
                .map(|item| {
                    let point_vec = item
                        .split(":")
                        .map(|x| x.parse::<i32>().unwrap())
                        .collect::<Vec<i32>>();
                    Point {
                        x: point_vec[0],
                        y: point_vec[1],
                    }
                })
                .collect();

            let slides = beat_info[6].parse::<u32>().unwrap();
            let length = beat_info[7].parse::<f64>().unwrap();

            slider = Some(Slider {
                curve_type,
                curve_points,
                trcurve_points: vec![],
                slides,
                length,
            });
        }

        let hitobj = OsuHitObject {
            pos: Point { x, y },
            trpos: None,
            time: t,
            hitobjecttype: hitobjecttype.clone(),
            hitsound: OsuHitSound::Normal,
            slider_params: slider,
            spinner_params: None,
        };

        // println!("{:?}", hitobj);
        // println!("{:?}", type_bits);
        // println!("{:?}", beat_info);

        osu_beatmap.hit_objects.push(hitobj);

        // println!("-----");
    }

    osu_beatmap.real_approach_time = 1.2;

    Ok(osu_beatmap)
}
