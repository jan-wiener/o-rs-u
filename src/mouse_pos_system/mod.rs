use crate::{WORLD_FG, WORLD_TOP, public_resources::*};
use bevy::prelude::*;
use bevy_vello::prelude::VelloSvg2d;
use bevy_window::CursorOptions;


pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_mouse_system);
        app.add_systems(Update, mouse_position_system);
    }
}

#[derive(Component)]
pub struct CursorImage;

fn init_mouse_system(mut commands: Commands, assets: Res<AssetServer>, copt: Option<Single<&mut CursorOptions>>, window: Single<&Window>,) {
    copt.unwrap().into_inner().visible = false;

    let image_handle = assets.load("skins/helpers/crosshair2.png");

    let cursor_image = Sprite::from_image(image_handle);

    let scale = 0.1 * (window.size().y / 1080.0);

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 999.0).with_scale(Vec3::splat(scale)),
        cursor_image,
        WORLD_FG,
        CursorImage,
        
    ));
    


}

fn mouse_position_system(
    time: Res<Time>,
    mut mouse_info: ResMut<MouseInfo>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    // mut posgui: Single<&mut Text, With<PosGUI>>,
    camera_s: Single<&Transform, With<Cameraz0>>,

    cimage: Option<Single<&mut Transform, (With<CursorImage>, Without<Cameraz0>)>>,
) {
    let Some(cursor_pos) = window.cursor_position() else {
        mouse_info.pressed = false;
        mouse_info.on_screen = false;
        return;
    };

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

    let mut cimage_inner = cimage.unwrap().into_inner();
    cimage_inner.translation = mouse_info.pos.extend(100.0);
}
