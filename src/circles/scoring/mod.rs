

use bevy::prelude::*;
use crate::public_resources::{ScoreInfo,HitScore,ScoreGui,AddScore, AccuracyGui};
use crate::osuparser::OsuBeatmap;


//Score = ((700000 * combo_bonus / max_combo_bonus) + (300000 * ((accuracy_percentage / 100) ^ 10) * elapsed_objects / total_objects) + spinner_bonus) * mod_multiplier




pub fn score_system(
    mut score_messages: ResMut<Messages<AddScore>>,
    mut score_info: ResMut<ScoreInfo>,
    osu: Res<OsuBeatmap>,
    mut score_gui: Single<&mut Text, With<ScoreGui>>,
    mut accuracy_gui: Single<&mut Text, (With<AccuracyGui>, Without<ScoreGui>)>,

) {


    for add_score in score_messages.drain() {
        score_info.hit_score.push(add_score.score.clone());

        let hit_score_number = add_score.score.to_number();

        println!("Score: {}", hit_score_number);
        // println!("combo: {} | accuracy: {} | objects: {}", score_info.current_combo,score_info.get_accuracy(), score_info.hit_score.len());
        

        if let HitScore::Miss = add_score.score {
            score_info.current_combo = 0;
        } else {
            score_info.current_combo += 1;
        }


        
        let score = osu.calculate_score(score_info.current_combo,score_info.get_accuracy(), score_info.hit_score.len());
        
        // println!("Score: {score}");
        
        score_gui.0 = format!("Score: {}", score);
        accuracy_gui.0 = format!("Accuracy: {}", (score_info.get_accuracy()*100.0).round() as usize)

        
        
    }


}








