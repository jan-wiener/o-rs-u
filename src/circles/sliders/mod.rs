use bevy::input::mouse;
use bevy::prelude::*;
use crate::public_resources::*;
use crate::osuparser::*;
use crate::vec_vec2_len;


pub fn tick_check(
    mut tick_check: ResMut<Messages<TickCheck>>,
    kb: Res<ButtonInput<KeyCode>>,
    mouse_info: Res<MouseInfo>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    osu: Res<OsuBeatmap>,
    mut add_score_msg: MessageWriter<AddScore>,

) {

    for tick in tick_check.drain() {
        let mut missed = false;
        if kb.pressed(KeyCode::KeyZ) || kb.pressed(KeyCode::KeyX) || mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
            if tick.trpos.distance(mouse_info.pos) <= osu.real_circle_size * 2.4 {
                add_score_msg.write(AddScore::new(tick.tick_type.to_hit_score()));
            } else {
                missed = true;
            }
        } else {
            missed = true;
        }
        if missed {
            // println!("MISSED A TICK");
            if let TickType::SliderEnd = tick.tick_type {
                continue;
            }
            add_score_msg.write(AddScore::new(HitScore::ComboMiss));
        }

    }

}





pub fn move_slider(
    mut tick_check: MessageWriter<TickCheck>,
    mut circles: Query<(&mut CircleInfo, &mut Transform)>,
    mut slider_res: ResMut<MovingSlidersRes>,
    time: Res<Time>,
    osu: Res<OsuBeatmap>,
    mut remove_circ: MessageWriter<RemoveCircle>,
    mut commands: Commands,
    bmw: Res<BeatmapWorkerInfo>,
) {
    let mut to_remove = vec![];
    // let time_since_start = ((time.elapsed_secs() - bmw.started_at) * 1000.0) as i32;

    for (idx, movingslider) in slider_res.sliders.iter_mut().enumerate() {
        if commands.get_entity(movingslider.entity).is_err() {
            to_remove.push(idx);
            continue;
        }

        let (mut circleinfo, mut tr) = circles.get_mut(movingslider.entity).unwrap();
        let points_inner = circleinfo.points.as_ref().unwrap();
        let length = vec_vec2_len(points_inner);

        // println!("ttc: {}\ndelta_slider: {}", osu.get_time_to_complete_slider(length), (time.elapsed_secs() - movingslider.started_at));

        let time_since_start = ((movingslider.started_at - bmw.started_at) * 1000.0) as i32;

        let t = (time.elapsed_secs() - movingslider.started_at)
            / osu.get_time_to_complete_slider(length, time_since_start);

        // println!("ttc: {}", osu.get_time_to_complete_slider(length, time_since_start));
        // println!("SLIDES: {}", circleinfo.slides);
        if t > (circleinfo.slides as f32) {
            let p = if circleinfo.slides % 2 != 0 {
                points_inner.last().unwrap().clone()
            } else {
                points_inner.first().unwrap().clone()
            };
            tick_check.write(TickCheck { trpos: p, tick_type: TickType::SliderEnd });
            remove_circ.write(RemoveCircle {
                entity: movingslider.entity,
            });
            continue;
        }
        

        let t_clamped = if t % 2.0 > 1.0 {
            2.0 - (t % 2.0)
        } else {
            t % 2.0
        };

        let mut new_pos: Option<Vec2> = None;
        let mut new_pos_idx = 0;

        let target_distance = length * t_clamped;
        let mut total = 0.0;
        let mut last: Vec2 = circleinfo.original_pos;
        for (point_idx, v) in points_inner.iter().enumerate() {
            total += last.distance(v.clone());
            last = v.clone();
            if total >= target_distance {
                new_pos_idx = point_idx;
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
        // println!(" npos {}", new_pos_idx);
        // println!("t: {} || slides: {} || Ticks: {:?}", t, circleinfo.slides, circleinfo.ticks);
        
        for i in circleinfo.last_pos_idx..new_pos_idx {
            if circleinfo.ticks.as_ref().unwrap().contains(&i) {
                tick_check.write(TickCheck { trpos: new_pos.clone(), tick_type: TickType::SliderTick });
            }
        }
        

        if t > (circleinfo.slides_completed as f32 + 1.0) {
            let p = if (circleinfo.slides_completed + 1) % 2 != 0 {
                points_inner.last().unwrap().clone()
            } else {
                points_inner.first().unwrap().clone()
            };
            tick_check.write(TickCheck { trpos: p, tick_type: TickType::SliderRepeat });
            circleinfo.slides_completed += 1;
        }
        circleinfo.last_pos_idx = new_pos_idx;

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








pub fn draw_tick(
    mut commands: Commands,
    mut drawline_msg: ResMut<Messages<DrawTick>>,
) {
    for dtick in drawline_msg.drain() {
        // println!("Drawing-- {:?}", line.points);  

        commands.spawn((
            Transform::from_translation(dtick.trpos.extend(900.0)),
                        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::splat(10.0)),
            
            SliderTick {
                timer: Timer::from_seconds(dtick.lifetime, TimerMode::Once),
            },
            
        ));
    }
}

pub fn remove_tick(time: Res<Time>, q: Query<(&mut SliderTick, Entity)>, mut commands: Commands) {
    for (mut slider_line, ent) in q {
        slider_line.timer.tick(time.delta());
        if slider_line.timer.is_finished() {
            commands.entity(ent).despawn();
        }
    }
}




