use bevy::prelude::*;
use crate::public_resources::*;
use crate::osuparser::*;
use crate::vec_vec2_len;






pub fn move_slider(
    mut circles: Query<(&CircleInfo, &mut Transform)>,
    mut slider_res: ResMut<MovingSlidersRes>,
    time: Res<Time>,
    osu: Res<OsuBeatmap>,
    mut remove_circ: MessageWriter<RemoveCircle>,
    mut commands: Commands,
    mut bmw: Res<BeatmapWorkerInfo>,
) {
    let mut to_remove = vec![];
    // let time_since_start = ((time.elapsed_secs() - bmw.started_at) * 1000.0) as i32;

    for (idx, movingslider) in slider_res.sliders.iter().enumerate() {
        if commands.get_entity(movingslider.entity).is_err() {
            to_remove.push(idx);
            continue;
        }

        let (circleinfo, mut tr) = circles.get_mut(movingslider.entity).unwrap();
        let points_inner = circleinfo.points.as_ref().unwrap();
        let length = vec_vec2_len(points_inner);

        // println!("ttc: {}\ndelta_slider: {}", osu.get_time_to_complete_slider(length), (time.elapsed_secs() - movingslider.started_at));

        let time_since_start = ((movingslider.started_at - bmw.started_at) * 1000.0) as i32;

        let mut t = (time.elapsed_secs() - movingslider.started_at)
            / osu.get_time_to_complete_slider(length, time_since_start);

        // println!("ttc: {}", osu.get_time_to_complete_slider(length, time_since_start));

        if t > (circleinfo.slides as f32) {
            remove_circ.write(RemoveCircle {
                entity: movingslider.entity,
            });
            continue;
        }
        t = if t % 2.0 > 1.0 {
            2.0 - (t % 2.0)
        } else {
            t % 2.0
        };

        let mut new_pos: Option<Vec2> = None;
        let target_distance = length * t;
        let mut total = 0.0;
        let mut last: Vec2 = circleinfo.original_pos;
        for v in points_inner {
            total += last.distance(v.clone());
            last = v.clone();
            if total >= target_distance {
                new_pos = Some(v.clone());
                break;
            }
        }
        let Some(new_pos) = new_pos else {
            remove_circ.write(RemoveCircle {
                entity: movingslider.entity,
            });
            continue;
        };

        tr.translation = new_pos.extend(tr.translation.z);
    }

    let mut offset = 0;
    for i in to_remove {
        slider_res.sliders.remove(i - offset);
        offset += 1;
    }
}

pub fn draw_from_points(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut drawline_msg: ResMut<Messages<DrawLine>>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    for line in drawline_msg.drain() {
        // println!("Drawing-- {:?}", line.points);

        let mut asset = GizmoAsset::default();

        asset.linestrip_2d(line.points, Color::srgb(0.0, 1.0, 0.0));

        let handle = gizmo_assets.add(asset);

        commands.spawn((
            Gizmo {
                handle,
                line_config: GizmoLineConfig {
                    width: 3.0,
                    ..default()
                },
                ..Default::default()
            },
            SliderLine {
                timer: Timer::from_seconds(line.lifetime, TimerMode::Once),
            },
            Transform::default(),
        ));
    }
}

pub fn remove_line(time: Res<Time>, q: Query<(&mut SliderLine, Entity)>, mut commands: Commands) {
    for (mut slider_line, ent) in q {
        slider_line.timer.tick(time.delta());
        if slider_line.timer.is_finished() {
            commands.entity(ent).despawn();
        }
    }
}