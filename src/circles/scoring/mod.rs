

use bevy::prelude::*;
use crate::AddScore;
use crate::ScoreInfo;



pub fn score_system(
    mut score_messages: ResMut<Messages<AddScore>>,
    mut score_info: ResMut<ScoreInfo>,

) {
    for add_score in score_messages.drain() {

    }

}




