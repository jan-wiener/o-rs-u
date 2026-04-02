

use bevy::prelude::*;
use crate::public_resources::{AccuracyGui, AddScore, ComboGui, HitScore, ScoreGui, ScoreInfo};
use crate::osuparser::OsuBeatmap;


//Score = ((700000 * combo_bonus / max_combo_bonus) + (300000 * ((accuracy_percentage / 100) ^ 10) * elapsed_objects / total_objects) + spinner_bonus) * mod_multiplier




pub fn score_system(
    mut score_messages: ResMut<Messages<AddScore>>,
    mut score_info: ResMut<ScoreInfo>,
    osu: Res<OsuBeatmap>,
    mut score_gui: Single<&mut Text, With<ScoreGui>>,
    mut accuracy_gui: Single<&mut Text, (With<AccuracyGui>, Without<ScoreGui>)>,
    mut combo_gui: Single<&mut Text, (With<ComboGui>, Without<ScoreGui>, Without<AccuracyGui>)>,

) {


    for add_score in score_messages.drain() {
        if add_score.score.affects_accuracy() {
            score_info.hit_score.push(add_score.score.clone());
        }
        

        let hit_score_number = add_score.score.to_number();

        println!("Score: {}", hit_score_number);
        // println!("combo: {} | accuracy: {} | objects: {}", score_info.current_combo,score_info.get_accuracy(), score_info.hit_score.len());
        

        if add_score.score.is_miss() {
            score_info.current_combo = 0;
        } else {
            score_info.current_combo += 1;
        }


        
        let score = osu.calculate_score(score_info.current_combo,score_info.get_accuracy(), score_info.hit_score.len());
        
        // println!("Score: {score}");
        
        score_gui.0 = format!("Score: {}", score);
        accuracy_gui.0 = format!("Accuracy: {:.2}%", (score_info.get_accuracy()*100.0));
        combo_gui.0 = format!("Combo: {}", score_info.current_combo);
        
        
    }


}








