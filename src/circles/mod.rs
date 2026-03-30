pub mod bezier;
pub mod catmull;
pub mod clicking;
pub mod etc;
pub mod rings;
pub mod scoring;
pub mod sliders;

use crate::circles::catmull::*;
use crate::circles::etc::*;
use crate::osuparser::*;
use crate::public_resources::*;
use bevy::prelude::*;

use crate::CIRCLE_VISUAL_MULTIPLIER;
use std::f32::consts::PI;

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
        let points: Option<Vec<Vec2>> = osuhitobj.points;
        let segments: Option<usize> = None;
        let slides: u32 = 1;

        match osuhitobj.hitobjecttype {
            OsuHitObjectType::Slider(_) => {
                println!("Ticks: {:?}", osuhitobj.ticks.unwrap());
                println!("Len: {}", osuhitobj.length);
                drawline_writer.write(DrawLine {
                    points: points.clone().unwrap(),
                    lifetime: osu.get_time_to_complete_slider(osuhitobj.length, time_since_start)
                        * slides as f32
                        + osu.real_approach_time,
                });
            }
            _ => {}
        }

        let radius = osu.get_real_circle_size() / general_info.real_circle_radius;

        let mut circletr =
            Transform::from_translation(pos.extend(900.0 - (time.elapsed_secs() / 1000.0)));
        circletr.scale = Vec3::splat(radius * CIRCLE_VISUAL_MULTIPLIER);
        // println!("CIRCLE tr: {}", circletr.translation);

        let mut centcmds = commands.spawn((
            circlemats.main.clone(),
            circlemats.main_mat.clone(),
            circletr,
            CircleInfo {
                moment_t: osuhitobj.time,
                original_pos: pos,
                clicked: false,
                size: osu.get_real_circle_size(),
                circle_type: osuhitobj.hitobjecttype,
                points,
                segments,
                slides,
            },
        ));

        // let mut tr = Transform::from_translation(ring.pos.extend(1.0));
        let mut tr = Transform::from_translation(Vec3::splat(0.0));
        tr.scale = Vec3::splat(2.0);

        let ring = OsuRing::new(2.0, osuhitobj.time);

        centcmds.with_child((
            circlemats.ring.clone(),
            circlemats.ring_mat.clone(),
            tr,
            ring,
        ));
    }
}
