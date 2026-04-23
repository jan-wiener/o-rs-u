use bevy::prelude::*;
use crate::public_resources::*;


pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mouse_position_system);
    }
}

fn mouse_position_system(
    time: Res<Time>,
    mut mouse_info: ResMut<MouseInfo>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    // mut posgui: Single<&mut Text, With<PosGUI>>,
    camera_s: Single<&Transform, With<Cameraz0> >,
) {
    if let Some(cursor_pos) = window.cursor_position() {
        mouse_info.on_screen = true;

        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
        let camera_tr = *camera_s;
        let camera_pos = &camera_tr.translation;
        let rel_pos = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
        let in_game_middle = window_size / 2.0;

        let mut in_game_pos = in_game_middle * rel_pos - Vec2::new(camera_pos.x, camera_pos.y);
        in_game_pos.y *= -1.0;

        let velocity = (in_game_pos - mouse_info.pos) / time.delta_secs();
        mouse_info.pos = in_game_pos;

        mouse_info.velocity = velocity;

        if !mouse_buttons.pressed(MouseButton::Left) {
            mouse_info.pressed = false;
        } else {
            mouse_info.pressed = true;
        }
    } else {
        mouse_info.pressed = false;
        mouse_info.on_screen = false;
    }
}
