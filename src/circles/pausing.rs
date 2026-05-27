use bevy::{prelude::*};

use crate::public_resources::{GameAudio, PauseMenu};

pub fn pausing_system(
    mut time: ResMut<Time<Virtual>>,
    kb: Res<ButtonInput<KeyCode>>,
    beatmap_music: Option<Single<&mut AudioSink, With<GameAudio>>>,
    pause_menu_items: Query<&mut Visibility, With<PauseMenu>>,
) {
    if !kb.just_pressed(KeyCode::Escape) {
        return;
    }
    let is_paused: bool = time.is_paused();
    match is_paused {
        true => {
            time.unpause();
        }
        false => {
            time.pause();
        }
    }
    if let Some(beatmap_music) = beatmap_music {
        beatmap_music.pause();
    }

    for mut item in pause_menu_items {
        *item = if is_paused {Visibility::Hidden} else {Visibility::Visible};
    }
}
