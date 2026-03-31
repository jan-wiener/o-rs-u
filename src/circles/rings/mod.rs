use bevy::prelude::*;

use crate::public_resources::*;
use crate::osuparser::*;


pub fn shrink_ring(
    time: Res<Time>,
    ring_q: Query<(&mut Transform, &mut OsuRing, &ChildOf)>,
    mut circle_q: Query<&mut CircleInfo>,
    mut rmcrc: MessageWriter<RemoveCircle>,
    osu: Res<OsuBeatmap>,
    mut slider_res: ResMut<MovingSlidersRes>,
    bmw: Res<BeatmapWorkerInfo>,
    mut add_score_msg: MessageWriter<AddScore>,
) {
    if time.elapsed_secs() < 1.5 {
        return;
    }

    for (mut tr, mut ring, ch) in ring_q {
        // println!("bmw time: {} | ring t: {} | realhitwindows s: {}", bmw.get_time_since_start(time.elapsed_secs()),ring.moment_t,osu.real_hit_window.score50);
        if tr.scale.x > 1.0 {
            // println!("Shrunk @ {}", time.elapsed_secs());
            tr.scale -= Vec3::splat(1.0) * time.delta_secs() / osu.real_approach_time;
        } else if !ring.slider_mode && (bmw.get_time_since_start(time.elapsed_secs()) - ring.moment_t) > osu.real_hit_window.score50 {
            let mut circle = circle_q.get_mut(ch.parent()).unwrap();
            circle.clicked = true;

            add_score_msg.write(AddScore::from_hit_score(&HitScore::Miss));
            

            match circle.circle_type {
                OsuHitObjectType::Circle(_) => {
                    rmcrc.write(RemoveCircle {
                        entity: ch.parent(),
                    });
                }
                OsuHitObjectType::Slider(_) => {
                    slider_res.sliders.push(MovingSlider {
                        entity: ch.parent(),
                        started_at: time.elapsed_secs(),
                        // target_slides: circle.slides,
                        done_slides: 0,
                    });
                    ring.slider_mode = true;
                }
                _ => {}
            }
        }
    }
}


