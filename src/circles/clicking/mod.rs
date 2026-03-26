use bevy::prelude::*;
use crate::public_resources::*;
use bevy::ecs::system::entity_command::despawn;
use crate::osuparser::*;




pub fn remove_circle(mut circles_to_remove: ResMut<Messages<RemoveCircle>>, mut commands: Commands) {
    for remove_circle_msg in circles_to_remove.drain() {
        commands
            .entity(remove_circle_msg.entity)
            .queue_silenced(despawn());
        // commands.entity(remove_circle_msg.entity).despawn();
    }
}

pub fn circle_click(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,

    mouse_info: Res<MouseInfo>,
    mut circles_q: Query<(&Transform, &mut CircleInfo, Entity, &Children)>,
    mut ring_q: Query<(&mut Transform, &mut OsuRing, &ChildOf), Without<CircleInfo>>,
    mut commands: Commands,
    mut removewriter: MessageWriter<RemoveCircle>,
    mut slider_res: ResMut<MovingSlidersRes>,
) {
    if !mouse_button.just_pressed(MouseButton::Left)
        && !kb.just_pressed(KeyCode::KeyX)
        && !kb.just_pressed(KeyCode::KeyY)
    {
        return;
    }

    println!("Clicked");

    let mut selected_ent: (Option<Entity>, f32) = (None, -100.0);

    for (tr, mut circleinfo, centity, children) in &mut circles_q {
        println!("{} || {}", mouse_info.pos, tr.translation.truncate());
        println!("size {}", circleinfo.size);
        if mouse_info.pos.distance(tr.translation.truncate()) <= circleinfo.size
            && selected_ent.1 < tr.translation.z
            && !circleinfo.clicked
        {
            selected_ent = (Some(centity), tr.translation.z);
        }
    }

    if let Some(ent) = selected_ent.0 {
        
        let (tr, mut circleinfo, entity, children) = circles_q.get_mut(ent).unwrap();
        circleinfo.clicked = true;
        match circleinfo.circle_type {
            // println!("___");
            OsuHitObjectType::Circle(_) => {
                removewriter.write(RemoveCircle { entity: ent });
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


