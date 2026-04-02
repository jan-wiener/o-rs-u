use bevy::prelude::*;
use bevy::ecs::system::entity_command::despawn;

use crate::osuparser::*;
use crate::public_resources::*;




pub fn remove_circle(mut circles_to_remove: ResMut<Messages<RemoveCircle>>, mut commands: Commands) {
    for remove_circle_msg in circles_to_remove.drain() {
        commands
            .entity(remove_circle_msg.entity)
            .queue_silenced(despawn());
        // commands.entity(remove_circle_msg.entity).despawn();
    }
}

pub fn circle_click(
    time: Res<Time<Fixed>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,

    mouse_info: Res<MouseInfo>,
    mut circles_q: Query<(&Transform, &mut CircleInfo, Entity, &Children)>,
    mut ring_q: Query<(&mut Transform, &mut OsuRing, &ChildOf), Without<CircleInfo>>,
    mut commands: Commands,
    mut removewriter: MessageWriter<RemoveCircle>,
    mut slider_res: ResMut<MovingSlidersRes>,
    bmw: Res<BeatmapWorkerInfo>,
    osu: Res<OsuBeatmap>,
    mut score: ResMut<ScoreInfo>,
    mut add_score_msg: MessageWriter<AddScore>,
) {
    
    let mut click_count = 0;
    if kb.just_pressed(KeyCode::KeyZ) || mouse_button.just_pressed(MouseButton::Left) {click_count +=1}
    if kb.just_pressed(KeyCode::KeyX) || mouse_button.just_pressed(MouseButton::Right) {click_count +=1}
    if click_count == 0 {
        return;
    }



    // println!("Clicked");


    let mut potential_entities: Vec<(Entity, f32)> = Vec::new();

    for (tr, mut circleinfo, centity, children) in &mut circles_q {

        if mouse_info.pos.distance(tr.translation.truncate()) <= circleinfo.size
            && !circleinfo.clicked
        {
            potential_entities.push((centity, tr.translation.z));
        }
    }
    potential_entities.sort_by(|a, b | {
        a.1.total_cmp(&b.1)
    });
    potential_entities.reverse();
    // let selected_entities = potential_entities.split_at;


    for (ent, _) in potential_entities.iter().take(click_count) {
        
        let (_tr, mut circleinfo, entity, children) = circles_q.get_mut(ent.to_owned()).unwrap();
        circleinfo.clicked = true;

        let delta = (bmw.get_time_since_start(time.elapsed_secs()) - circleinfo.moment_t).abs();

        let result = HitScore::from_delta(delta, &osu.real_hit_window);


        //   Score = Hit Value + (Hit Value * ((Combo multiplier * Difficulty multiplier * Mod multiplier) / 25))
        
        add_score_msg.write(AddScore::new(result));
        


        match circleinfo.circle_type {
            // println!("___");
            OsuHitObjectType::Circle(_) => {
                removewriter.write(RemoveCircle { entity: ent.clone() });
            }
            OsuHitObjectType::Slider(_) => {
                slider_res.sliders.push(MovingSlider {
                    entity,
                    started_at: time.elapsed_secs(),
                    // target_slides: circleinfo.slides,
                    done_slides: 0,
                });
                ring_q
                    .get_mut(children.first().unwrap().to_owned())
                    .unwrap()
                    .1
                    .slider_mode = true;
            }
            _ => {}
        }
    }
}


