use bevy::{prelude::*};

use crate::public_resources::GameAudio;

pub fn pausing_system(
    mut time: ResMut<Time<Virtual>>,
    kb: Res<ButtonInput<KeyCode>>,
    beatmap_music: Option<Single<&mut AudioSink, With<GameAudio>>>,
) {
    if !kb.just_pressed(KeyCode::Escape) {
        return;
    }
    match time.is_paused() {
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
}
