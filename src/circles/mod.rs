pub mod bezier;
pub mod catmull;
pub mod clicking;
pub mod etc;
pub mod pausing;
pub mod rings;
pub mod scoring;
pub mod sliders;

use crate::SVG_MODE;
use crate::WORLD_BG;
use crate::WORLD_FG;
use crate::osuparser::*;
use crate::public_resources::*;
use bevy::prelude::*;
use bevy_vello::prelude::VelloSvg2d;
use bevy_vello::prelude::VelloSvgAnchor;

use crate::CIRCLE_VISUAL_MULTIPLIER;

pub fn summon_circle(
    circlemats: Res<CircleMaterials>,
    osu: Res<OsuBeatmap>,
    _window: Single<&Window>,
    mut circles_to_summon: ResMut<Messages<OsuHitObject>>,
    mut drawline_writer: MessageWriter<DrawLine>,
    mut drawtickwriter: MessageWriter<DrawTick>,
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    bmw: Res<BeatmapWorkerInfo>,
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
        let points: Option<Vec<Vec2>> = osuhitobj.points;
        let ticks: Option<Vec<usize>> = osuhitobj.ticks;
        let segments: Option<usize> = None;
        let slides = osuhitobj.slides;

        match osuhitobj.hitobjecttype {
            OsuHitObjectType::Slider(_) => {
                let lifetime = osu.get_time_to_complete_slider(osuhitobj.length, time_since_start)
                    * slides as f32
                    + osu.real_approach_time;
                for tick_idx in ticks.as_ref().unwrap() {
                    let tick_pos = points.as_ref().unwrap()[*tick_idx];
                    drawtickwriter.write(DrawTick {
                        trpos: tick_pos.clone(),
                        lifetime,
                    });
                }

                // println!("Ticks: {:?}", osuhitobj.ticks.unwrap());
                // println!("Len: {}", osuhitobj.length);
                drawline_writer.write(DrawLine {
                    points: points.clone().unwrap(),
                    lifetime,
                });
            }
            _ => {}
        }

        let radius = osu.get_real_circle_size() / general_info.real_circle_radius;

        let mut circletr = Transform::from_translation(
            pos.extend(900.0 - ((time.elapsed_secs() - bmw.started_at) / 1000.0)),
        );
        circletr.scale = Vec3::splat(radius * CIRCLE_VISUAL_MULTIPLIER);
        // println!("CIRCLE tr: {}", circletr.translation);

        let mut centcmds: EntityCommands<'_>;

        if SVG_MODE {
            let vello_svg_handle = if let OsuHitObjectType::Slider(_) = osuhitobj.hitobjecttype {
                circlemats.slider_svg.clone()
            } else {
                circlemats.main_svg.clone()
            };
            centcmds = commands.spawn((
                circletr,
                Visibility::Visible,
                CircleInfo {
                    moment_t: osuhitobj.time,
                    original_pos: pos,
                    clicked: false,
                    ticks,
                    size: osu.get_real_circle_size(),
                    circle_type: osuhitobj.hitobjecttype,
                    points,
                    segments,
                    slides,
                    slides_completed: 0,
                    last_pos_idx: 0,
                },
            ));
            
            centcmds.with_child((
                VelloSvg2d(vello_svg_handle),
                VelloSvgAnchor::Center,
                Transform::from_scale(Vec3::splat(1.03) * osu.screen_size.y / 1080.0),
                WORLD_FG,
            ));
        } else {
            centcmds = commands.spawn((
                circlemats.main.clone(),
                circlemats.main_mat.clone(),
                circletr,
                Visibility::Visible,
                CircleInfo {
                    moment_t: osuhitobj.time,
                    original_pos: pos,
                    clicked: false,
                    ticks,
                    size: osu.get_real_circle_size(),
                    circle_type: osuhitobj.hitobjecttype,
                    points,
                    segments,
                    slides,
                    slides_completed: 0,
                    last_pos_idx: 0,
                },
                WORLD_FG,
            ));
        }

        // let mut tr = Transform::from_translation(ring.pos.extend(1.0));
        let mut tr = Transform::from_translation(Vec3::splat(0.0));
        tr.scale = Vec3::splat(2.0);

        let ring = OsuRing::new(2.0, osuhitobj.time);

        centcmds.with_child((
            circlemats.ring.clone(),
            circlemats.ring_mat.clone(),
            tr,
            Visibility::Visible,
            ring,
            WORLD_BG,
        ));
    }
}

pub fn change_material_system(
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut MeshMaterial2d<ColorMaterial>, &OsuRing)>,
    bmw: Res<BeatmapWorkerInfo>,
    time: Res<Time>,
    osu: Res<OsuBeatmap>,
    circlemats: Res<CircleMaterials>,
) {
    for (mut material_handle, circleinfo) in &mut query {
        // let Some(material) = materials.get_mut(material_handle.0.id()) else {
        //     error!("Material not found!");
        //     continue;
        // };
        let delta = (bmw.get_time_since_start(time.elapsed_secs()) - circleinfo.moment_t).abs();
        let result = HitScore::from_delta(delta, &osu.real_hit_window);

        match result {
            HitScore::Miss => {
                *material_handle = circlemats.main_mat.clone();
            }
            HitScore::Meh => {
                *material_handle = circlemats.meh_mat.clone();
            }
            HitScore::Ok => {
                *material_handle = circlemats.ok_mat.clone();
            }
            HitScore::Great => {
                *material_handle = circlemats.great_mat.clone();
            }
            _ => {}
        }
    }
}

pub fn what_should_i_click(_commands: Commands, circles: Query<&CircleInfo>, mut marker: Single<(&mut Transform, &mut Visibility), With<WhatShouldIClick>>) {
    let mut first_circle: Option<(&CircleInfo, f32)> = None;
    for circle in circles {
        if circle.clicked {
            continue;
        }
        if let Some(first_circle_inner) = first_circle {
            if circle.moment_t < first_circle_inner.1{
                first_circle = Some((circle, circle.moment_t));
            }
        } else {
            first_circle = Some((circle, circle.moment_t));
        }
    }

    if let Some(first_circle_inner) = first_circle {
        *marker.1 = Visibility::Visible;
        marker.0.translation = first_circle_inner.0.original_pos.extend(960.0);
    } else {
        *marker.1 = Visibility::Hidden;
    }




}
