

use bevy::prelude::*;
use crate::AddScore;
use crate::ScoreInfo;
use crate::osuparser::OsuBeatmap;


//Score = ((700000 * combo_bonus / max_combo_bonus) + (300000 * ((accuracy_percentage / 100) ^ 10) * elapsed_objects / total_objects) + spinner_bonus) * mod_multiplier




pub fn score_system(
    mut score_messages: ResMut<Messages<AddScore>>,
    mut score_info: ResMut<ScoreInfo>,
    osu: Res<OsuBeatmap>,

) {
    for add_score in score_messages.drain() {
        println!("Score: {}", add_score.score);
        if add_score.score == 0 {
            score_info.current_combo = 0;
        } else {
            score_info.current_combo += 1;
        }
        let score = osu.calculate_score(score_info.current_combo,score_info.get_accuracy(), score_info.hit_score.len());
        
    }


}




