use std::{error::Error, fs, io::Read, path::Path};

use bevy::{math::f32, prelude::*, reflect::GetTupleField};

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
    pub slides: usize,
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

    pub ticks: Option<Vec<usize>>,
    pub points: Option<Vec<Vec2>>,
    pub segments: Option<usize>,
    pub slides: usize,
    pub length: f32,
}

use crate::circles::etc;
use crate::circles::bezier;
use crate::circles::catmull;
use std::f32::consts::PI;
use crate::circles::etc::*;

impl OsuHitObject {
    fn compute_points(&mut self) {

        let pos = self.trpos.expect("Not converted pos");
        let slider_info = self.slider_params.as_ref().unwrap();

        self.slides = slider_info.slides;
        // let tr_curve_points_inner = slider_info.trcurve_points;
        let mut points_inner: Vec<Vec2> = vec![];

        // let iter_steps = (0..10).map(|x| x as f32 / 10.0);
        let mut length = 0.0;

        let mut linear = || {
            let mut key_points = slider_info.trcurve_points.clone();

            *(&mut length) = etc::vec_vec2_len(&key_points);
            let steps = length / 10.0;
            println!("{:?}", key_points);
            for (idx, key_point) in key_points.iter().enumerate() {
                if idx.checked_sub(1).is_some()
                    && let Some(last) = key_points.get(idx - 1)
                {
                    let seg_length = key_point.distance(*last);
                    let seg_steps = seg_length / length * steps;
                    let seg_step_length = seg_length / seg_steps;

                    println!("{}, {}, {}", seg_length, seg_steps, seg_step_length);

                    let mut seg_length_covered = 0.0;
                    let mut next = last.clone();
                    loop {
                        let step = (key_point - last).normalize() * seg_step_length;
                        next = next + step;
                        seg_length_covered += step.length();
                        if seg_length_covered > seg_length {
                            println!("broken");
                            break;
                        }
                        points_inner.push(next);
                    }
                }
            }
        };
        // println!("t: {:?}", slider_info.curve_type);
        match slider_info.curve_type {
            CurveType::Bezier => {
                let mut splits: Vec<Vec<Vec2>> = vec![vec![]];
                for (idx, trpoint) in slider_info.trcurve_points.iter().enumerate() {
                    if idx.checked_sub(1).is_some()
                        && let Some(last) = slider_info.trcurve_points.get(idx - 1)
                    {
                        if last == trpoint {
                            splits.push(vec![]);
                        }
                    }
                    splits.last_mut().unwrap().push(trpoint.clone());
                }

                for split in &splits {
                    length += bezier::bezier_length(split, 10);
                }
                // length = bezier_length(&slider_info.trcurve_points, 10);

                // println!("splits: {}", splits.len());

                let steps = (length / 10.0 / splits.len() as f32) as i32;
                self.segments = Some(splits.len());
                for split in splits {
                    for step in (0..steps).map(|x| x as f32 / steps as f32) {
                        points_inner.push(bezier::bezier_casteljau(&split, step));
                    }
                }
                length = etc::vec_vec2_len(&points_inner);

                // println!("len: {}\nPoints: {:?}", length, points_inner);
            }
            CurveType::Centripetal => {
                let spline = catmull::CentripetalCatmullRomSpline2d::new(&slider_info.trcurve_points);
                length = spline.length(10 / spline.segments.len());
                let steps = (length / 10.0) as i32;
                for step in (0..steps).map(|x| x as f32 / steps as f32) {
                    points_inner.push(bezier::bezier_casteljau(
                        &slider_info.trcurve_points,
                        step * (spline.segments.len() as f32),
                    ));
                }
            }
            CurveType::Linear => {
                linear();
                // let mut key_points = slider_info.trcurve_points.clone();
                // length = vec_vec2_len(&key_points);
                // let steps = length / 10.0;
                // println!("{:?}", key_points);
                // for (idx, key_point) in key_points.iter().enumerate() {
                //     if idx.checked_sub(1).is_some()
                //         && let Some(last) = key_points.get(idx - 1)
                //     {
                //         let seg_length = key_point.distance(*last);
                //         let seg_steps = seg_length / length * steps;
                //         let seg_step_length = seg_length / seg_steps;
                //         println!("{}, {}, {}", seg_length, seg_steps, seg_step_length);
                //         let mut seg_length_covered = 0.0;
                //         let mut next = last.clone();
                //         loop {
                //             let step = (key_point - last).normalize() * seg_step_length;
                //             next = next + step;
                //             seg_length_covered += step.length();
                //             if seg_length_covered > seg_length {
                //                 println!("broken");
                //                 break;
                //             }
                //             points_inner.push(next);
                //         }
                //     }
                // }
                // println!("{:?}", points_inner);
            }
            CurveType::PerfectCircle => {
                let key_points = slider_info.trcurve_points.clone();
                // println!("key points: {:?}", key_points);
                let start = key_points[0];
                let control = key_points[1];
                let end = key_points[2];

                let control_start = start - control;
                let control_end = end - control;

                let start_control_middle = control + (control_start * 0.5);
                let end_control_middle = control + (control_end * 0.5);

                let start_control_perpendicular =
                    (-(control_start / 2.0)).rotate(Vec2::from_angle(PI * 0.5));
                let end_control_perpendicular =
                    (-(control_end / 2.0)).rotate(Vec2::from_angle(PI * 0.5));

                let Some(center) = line_intersection(
                    start_control_middle,
                    start_control_perpendicular.normalize(),
                    end_control_middle,
                    end_control_perpendicular.normalize(),
                ) else {
                    println!("Break, line fail");
                    linear();
                    return;
                };

                let radius = center.distance(start);

                let center_start = start - center;
                let center_control = control - center;
                let center_end = end - center;

                let alpha = norm_angle(center_start.to_angle());
                let beta = norm_angle(center_control.to_angle());
                let gamma = norm_angle(center_end.to_angle());
                let d_se = ccw_diff(alpha, gamma);
                let d_sc = ccw_diff(alpha, beta);
                let counter_clockwise = d_sc <= d_se;

                let mut current = center_start;

                // println!("{:?},{:?},{:?},{:?}",start_control_middle,
                //     start_control_perpendicular.normalize(),
                //     end_control_middle,
                //     end_control_perpendicular.normalize(),
                // );
                // println!("alpha: {alpha}; beta: {beta}; gamma: {gamma}");
                // println!("pos: {:?}, center: {:?}", start, center);
                println!("Current: {} | gamma: {}", norm_angle(current.to_angle()), gamma); 
                let angle_step = PI / radius;
                let mut current_angle = alpha;
                
                if counter_clockwise {
                    println!("delta bigger");
                    while ccw_diff(current_angle, gamma) > angle_step {
                        current_angle += angle_step;
                        current = current.rotate(Vec2::from_angle(angle_step));
                        points_inner.push(center + current.clone());
                    }
                } else {
                    println!("delta smaller");
                    while ccw_diff(gamma, current_angle) > angle_step {
                        current_angle -= angle_step;
                        current = current.rotate(Vec2::from_angle(-angle_step));
                        points_inner.push(center + current.clone());
                    }
                }
                length = vec_vec2_len(&points_inner);
                // println!("PERFECT CIRCLE POINTS: {:?}", points_inner);
            }
        }
        // println!("{}", osu.get_time_to_complete_slider(length));
        // length = vec_vec2_len(&points_inner);
        println!("Points inner: {:?}", points_inner);
        self.points = Some(points_inner);
        self.length = length;
    }
}

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

#[derive(Default)]
pub struct RealHitWindow {
    pub score300: f32,
    pub score100: f32,
    pub score50: f32,
}

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
    fn from_tuple(t: (i32, f32, i32, i32, i32, i32, bool, i32)) -> Self {
        OsuTimingPoint::new(t.0, t.1, t.2, t.3, t.4, t.5, t.6, t.7)
    }
}

#[derive(Resource, Default)]
pub struct OsuBeatmap {
    pub hit_objects: Vec<OsuHitObject>,
    pub difficulty: OsuDifficulty,

    pub timing_points: Vec<OsuTimingPoint>,

    pub real_circle_size: f32,
    pub real_hit_window: RealHitWindow,
    pub real_approach_time: f32,
    pub max_combo: usize,

    pub difficulty_score_multiplier: f32,

    pub screen_size: Vec2,
}

const DEFAULT_OSU_TIMING_POINT: OsuTimingPoint = OsuTimingPoint {
    time: 0,
    beat_length: 600.0,
    meter: 4,
    sample_set: 0,
    sample_index: 0,
    volume: 0,
    uninherited: true,
    effects: 0,
};

impl OsuBeatmap {


    //Score = ((700000 * combo_bonus / max_combo_bonus) + (300000 * ((accuracy_percentage / 100) ^ 10) * elapsed_objects / total_objects) + spinner_bonus) * mod_multiplier

    pub fn calculate_score(&self, combo: usize, accuracy: f32, elapsed_objects: usize) -> usize {
        ((700000.0 * (combo as f32) / (self.max_combo as f32)) + (300000.0 * accuracy.powi(10) * (elapsed_objects as f32) / (self.hit_objects.len() as f32) )) as usize
    }
    
    pub fn calc_max_combo(&self) -> usize {
        let mut max_combo = 0;
        for osuhitobj in &self.hit_objects {
            match osuhitobj.hitobjecttype {
                OsuHitObjectType::Slider(_) => {
                    max_combo += 2 + osuhitobj.ticks.as_ref().unwrap().len();
                },
                OsuHitObjectType::Circle(_) => {
                    max_combo += 1;
                },
                _ => {}
            } 
        }
        max_combo
    }

    pub fn calc_real_values(&mut self) {
        let score300 = (80.0 - 6.0 * self.difficulty.overall_diff) / 1000.0;
        let score100 = (140.0 - 8.0 * self.difficulty.overall_diff) / 1000.0;
        let score50 = (200.0 - 10.0 * self.difficulty.overall_diff) / 1000.0;

        self.real_hit_window = RealHitWindow {
            score300,
            score100,
            score50,
        };

        self.real_circle_size =
            (self.screen_size.y / 480.0) * (54.4 - 4.48 * (self.difficulty.circle_size as f32));

        if self.difficulty.approach_rate < 5.0 {
            self.real_approach_time = 1200.0 + 120.0 * (5.0 - self.difficulty.approach_rate);
        } else {
            self.real_approach_time = 1200.0 - 150.0 * (self.difficulty.approach_rate - 5.0);
        }
        self.real_approach_time /= 1000.0;

        let difficulty_points =
            self.difficulty.circle_size + self.difficulty.hp_drain + self.difficulty.overall_diff;
        let diff_multiplier = (difficulty_points / 6.0).floor() + 2.0;
        self.difficulty_score_multiplier = diff_multiplier;

        

        for osuhitobj_idx in 0..self.hit_objects.len() {
            println!("{:?}", self.hit_objects[osuhitobj_idx]);
            if let OsuHitObjectType::Slider(_) = self.hit_objects[osuhitobj_idx].hitobjecttype {
                // println!("Slider points: {:?}", self.hit_objects[osuhitobj_idx].points);
                let tick_interval = (self.get_beat_length((self.hit_objects[osuhitobj_idx].time*1000.0) as i32) / 1000.0) / self.difficulty.slider_tick_rate;

                self.hit_objects[osuhitobj_idx].compute_points();

                // println!("-----");
                // println!("Tick interval {}", tick_interval);

                let ttc = self.get_time_to_complete_slider(self.hit_objects[osuhitobj_idx].length, (self.hit_objects[osuhitobj_idx].time * 1000.0) as i32);
                let length_per_tick = self.hit_objects[osuhitobj_idx].length * (tick_interval / ttc);


                let mut passed_length = 0.0; 
                let mut ticks: Vec<usize> = vec![];
                let points = self.hit_objects[osuhitobj_idx].points.as_ref().unwrap();
                
                let mut last: &Vec2 = &points[0];
                for (pidx, point) in points.iter().enumerate() {
                    // println!("dist: {} | l: {}", passed_length, (length_per_tick * ((ticks.len() + 1) as f32)));;
                    passed_length += point.distance(*last);
                    last = point;
                    if passed_length > (length_per_tick * ((ticks.len() + 1) as f32)) {
                        ticks.push(pidx);
                    }

                }
                

                if ticks.len() > 0 && vec_vec2_len(&points[ticks.last().unwrap().to_owned()..points.len()]) < (length_per_tick / 7.5) {
                    ticks.pop();
                }
                println!("len: {} | lpt: {} | ticks: {}", self.hit_objects[osuhitobj_idx].length, length_per_tick, ticks.len());

                self.hit_objects[osuhitobj_idx].ticks = Some(ticks);
            }
        }


        let mut ticks = 0;
        for osuhitobj in &self.hit_objects {
            if let OsuHitObjectType::Slider(_) = osuhitobj.hitobjecttype {
                ticks += osuhitobj.ticks.as_ref().unwrap().len() * osuhitobj.slides;
            }
        }
        println!("Ticks: {}", ticks);
        
        // println!("Max Combo: {}", self.calc_max_combo());
        self.max_combo = self.calc_max_combo();


    }

    pub fn get_current_timing_points(
        &self,
        time_since_start: i32,
    ) -> (OsuTimingPoint, Option<OsuTimingPoint>) {
        let mut last_uninherited: &OsuTimingPoint = &DEFAULT_OSU_TIMING_POINT;
        let mut last_inherited_opt_ref: Option<&OsuTimingPoint> = None;
        for (idx, timing) in self.timing_points.iter().enumerate() {
            if timing.time < time_since_start {
                if timing.uninherited {
                    last_uninherited = timing;
                    last_inherited_opt_ref = None;
                } else {
                    last_inherited_opt_ref = Some(timing);
                }
            } else {
                break;
            }
        }
        let last_inherited: Option<OsuTimingPoint> = if let Some(inherited) = last_inherited_opt_ref
        {
            Some(inherited.to_owned())
        } else {
            None
        };
        (last_uninherited.to_owned(), last_inherited)
    }

    pub fn get_current_slider_velocity(&self, time_since_start: i32) -> f32 {
        let (uninherited, inherited) = self.get_current_timing_points(time_since_start);
        if let Some(inherited_inner) = inherited {
            return (-100.0 / inherited_inner.beat_length);
        } else {
            return 1.0;
        }
    }

    pub fn get_beat_length(&self, time_since_start: i32) -> f32 {
        self.get_current_timing_points(time_since_start)
            .0
            .beat_length
    }

    pub fn get_time_to_complete_slider(&self, length: f32, time_since_start: i32) -> f32 {
        // println!("current SV: {}\nSM: {}\nbeat length: {}\ntss: {time_since_start}", self.get_current_slider_velocity(time_since_start), self.difficulty.slider_multiplier, self.get_beat_length(time_since_start));
        return (((length * (480.0 / self.screen_size.y))
            / (self.difficulty.slider_multiplier
                * 100.0
                * self.get_current_slider_velocity(time_since_start)))
            * self.get_beat_length(time_since_start))
            / 1000.0;
    }

    pub fn get_real_circle_size(&self) -> f32 {
        let x = 54.4 - 4.48 * (self.difficulty.circle_size as f32);
        (self.screen_size.y / 480.0) * x
    }
}

fn str_to_line_vec(s: &str) -> Vec<String> {
    s.trim()
        .to_string()
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
    timing_points.pop();

    difficulty = difficulty
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect();
    timing_points = timing_points
        .into_iter()
        .filter(|item| !item.is_empty())
        .collect();

    let beatmap_str = split_s[8].trim();
    let beatmap_vec = str_to_line_vec(split_s[8].as_str());

    let mut osu_beatmap = OsuBeatmap::default();

    let difficulty_split = difficulty
        .iter()
        .map(|org| {
            let items = org
                .split(":")
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            (items[0].to_owned(), items[1].parse::<f32>().unwrap())
        })
        .collect::<Vec<(String, f32)>>();

    for (name, value) in difficulty_split {
        match name.as_str() {
            "HPDrainRate" => osu_beatmap.difficulty.hp_drain = value,
            "CircleSize" => osu_beatmap.difficulty.circle_size = value,
            "OverallDifficulty" => osu_beatmap.difficulty.overall_diff = value,
            "ApproachRate" => osu_beatmap.difficulty.approach_rate = value,
            "SliderMultiplier" => osu_beatmap.difficulty.slider_multiplier = value,
            "SliderTickRate" => osu_beatmap.difficulty.slider_tick_rate = value,
            _ => {}
        }
    }
    println!("diffs: {:?}", osu_beatmap.difficulty);

    let timing_points_split: Vec<Vec<String>> = timing_points
        .into_iter()
        .map(|item| item.split(",").map(|s| s.to_string()).collect())
        .collect();

    let mut timing_points_real: Vec<OsuTimingPoint> = vec![];
    for timing_point_string_vec in timing_points_split {
        // let mut timing_point: OsuTimingPoint = OsuTimi
        let mut x: (i32, f32, i32, i32, i32, i32, bool, i32) = (0, 0.0, 0, 0, 0, 0, false, 0);
        println!("timing point {:?}", timing_point_string_vec);
        for (i, item) in timing_point_string_vec.into_iter().enumerate() {
            if i == 1 {
                *(x.get_field_mut(i).unwrap()) = item.parse::<f32>().unwrap();
            } else if i == 6 {
                *(x.get_field_mut(i).unwrap()) = item.parse::<i32>().unwrap() == 1;
            } else {
                *(x.get_field_mut(i).unwrap()) = item.parse::<i32>().unwrap();
            }
        }
        timing_points_real.push(OsuTimingPoint::from_tuple(x));
    }
    osu_beatmap.timing_points = timing_points_real;

    // println!("{:?}", timing_points_real);

    // for i in timings_split {
    //     println!("{}", i.len());
    // }
    // println!("timings: {:?}", timing_points_split);

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

            // println!("CurveType: {}", curve_type_s);

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

            let slides = beat_info[6].parse::<usize>().unwrap();
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
            ..Default::default()
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
