mod catmull;
mod bezier;
pub mod clicking;
pub mod etc;
pub mod sliders;
pub mod rings;


use crate::circles::catmull::*;
use crate::public_resources::*;
use crate::osuparser::*;
use crate::circles::etc::*;
use bevy::prelude::*;

use std::f32::consts::PI;
use crate::CIRCLE_VISUAL_MULTIPLIER;


pub fn summon_circle(
    circlemats: Res<CircleMaterials>,
    osu: Res<OsuBeatmap>,
    window: Single<&Window>,
    mut circles_to_summon: ResMut<Messages<OsuHitObject>>,
    mut drawline_writer: MessageWriter<DrawLine>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut bmw: Res<BeatmapWorkerInfo>,
    time: Res<Time>,
    general_info: Res<GeneralInfo>,
) {
    let time_since_start = ((time.elapsed_secs() - bmw.started_at) * 1000.0) as i32;

    for osuhitobj in circles_to_summon.drain() {
        // println!("OsuHitObject found");

        let pos = osuhitobj.trpos.expect("somebody didnt convert pos");

        if !osuhitobj.hitobjecttype.is_circle_like() {
            continue;
        }

        // let size = osu.get_real_circle_size(window.size());
        let mut points: Option<Vec<Vec2>> = None;
        let mut segments: Option<usize> = None;
        let mut slides: u32 = 1;

        match osuhitobj.hitobjecttype {
            OsuHitObjectType::Slider(_) => {
                let slider_info = osuhitobj.slider_params.unwrap();

                slides = slider_info.slides;
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
                        segments = Some(splits.len());
                        for split in splits {
                            for step in (0..steps).map(|x| x as f32 / steps as f32) {
                                points_inner.push(bezier::bezier_casteljau(&split, step));
                            }
                        }
                        length = etc::vec_vec2_len(&points_inner);

                        // println!("len: {}\nPoints: {:?}", length, points_inner);
                    }
                    CurveType::Centripetal => {
                        let spline =
                            CentripetalCatmullRomSpline2d::new(&slider_info.trcurve_points);
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
                            break;
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

                        if counter_clockwise {
                            println!("delta bigger");
                            while norm_angle(current.to_angle()) < gamma {
                                current = current.rotate(Vec2::from_angle(PI / radius));
                                points_inner.push(center + current.clone());
                            }
                        } else {
                            println!("delta smaller");
                            while norm_angle(current.to_angle()) > gamma {
                                current = current.rotate(Vec2::from_angle(-PI / radius));
                                points_inner.push(center + current.clone());
                            }
                        }
                        length = vec_vec2_len(&points_inner);
                        // println!("PERFECT CIRCLE POINTS: {:?}", points_inner);
                    }
                    _ => {
                        warn!("Curve for a slider not loaded!")
                    }
                }
                // println!("{}", osu.get_time_to_complete_slider(length));
                drawline_writer.write(DrawLine {
                    points: points_inner.clone(),
                    lifetime: osu.get_time_to_complete_slider(length, time_since_start)
                        * slides as f32
                        + osu.real_approach_time,
                });
                points = Some(points_inner);
            }
            _ => {}
        }


        let radius = osu.get_real_circle_size() / general_info.real_circle_radius;

        let mut circletr = Transform::from_translation(pos.extend(900.0-(time.elapsed_secs()/1000.0)));
        circletr.scale =
            Vec3::splat(radius*CIRCLE_VISUAL_MULTIPLIER);
        // println!("CIRCLE tr: {}", circletr.translation);

        let mut centcmds = commands.spawn((
            circlemats.main.clone(),
            circlemats.main_mat.clone(),
            circletr,
            CircleInfo {
                original_pos: pos,
                clicked: false,
                size: osu.get_real_circle_size() ,
                circle_type: osuhitobj.hitobjecttype,
                points,
                segments,
                slides,
            },
        ));

        // let mut tr = Transform::from_translation(ring.pos.extend(1.0));
        let mut tr = Transform::from_translation(Vec3::splat(0.0));
        tr.scale = Vec3::splat(2.0);

        let ring = OsuRing::new(2.0);

        centcmds.with_child((
            circlemats.ring.clone(),
            circlemats.ring_mat.clone(),
            tr,
            ring,
        ));
    }
}