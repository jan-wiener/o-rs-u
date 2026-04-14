use bevy::prelude::*;
use crate::public_resources::*;
use crate::osuparser::*;
use std::path::Path;



pub fn load_osu_beatmap(
    mut bmap_msg: ResMut<Messages<LoadBeatmap>>,
    window: Single<&Window>,
    mut osu: ResMut<OsuBeatmap>,
    mut bmw: ResMut<BeatmapWorkerInfo>,
    time: Res<Time>,
    mut score_info: ResMut<ScoreInfo>,
    mut general_info: ResMut<GeneralInfo>,
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut audio: Single<Entity, With<GameAudio>>,
    // mut playing_audio_opt: Option<Single<(&mut AudioSink), With<GameAudio>>>,
) {
    let mut bmap_info_opt: Option<LoadBeatmap> = None;

    for lbmp in bmap_msg.drain() {
        bmap_info_opt = Some(lbmp);
    }

    let Some(bmap_info) = bmap_info_opt else {
        return;
    };
    info!("Beatmap LOADING...");

    score_info.accuracy = 1.0;
    score_info.score = 0;

    let screen_size = window.size();

    let mut beatmap = parse_osu_file(Path::new(&bmap_info.path)).unwrap();

    for hitobj in &mut beatmap.hit_objects {
        hitobj.trpos = Some(hitobj.pos.to_real_pos(screen_size));
        if let Some(slider) = &mut hitobj.slider_params {
            slider.trcurve_points.push(hitobj.trpos.unwrap());
            let mut tap = slider
                .curve_points
                .iter()
                .map(|point| point.to_real_pos(screen_size))
                .collect::<Vec<Vec2>>();
            slider.trcurve_points.append(&mut tap);
            // println!(
            //     "realpos: {}, \ntrcurvepoints: {:?}",
            //     hitobj.trpos.unwrap(),
            //     slider.trcurve_points
            // );
        }
    }
    beatmap.screen_size = screen_size;
    beatmap.calc_real_values();
    println!("{}", beatmap.real_approach_time);
    info!("Beatmap FINISHED LOADING...");
    *osu = beatmap;
    bmw.started_at = time.elapsed_secs() + 3.0;
    bmw.start = true;



    // audio.0.0 = asset_server.load("beatmaps/audio.ogg");
    // audio.1.paused = true;
    commands.entity(*audio).despawn();
    commands.spawn((
        GameAudio,
        AudioPlayer::new(asset_server.load(bmap_info.audio)),
        PlaybackSettings::ONCE.paused()
    ));

}


pub fn play_audio(
    mut audio: Single<&mut AudioSink, With<GameAudio>>,
    time: Res<Time<Virtual>>,
    mut bmw: ResMut<BeatmapWorkerInfo>,
) {
    let time_since_start = time.elapsed_secs() - bmw.started_at;
    if time_since_start > 0.0 && audio.is_paused() && !time.is_paused() {
        audio.play();
    }
}



pub fn beatmap_worker(
    mut oho_msg: MessageWriter<OsuHitObject>,
    mut osu: ResMut<OsuBeatmap>,
    time: Res<Time>,
    mut bmw: ResMut<BeatmapWorkerInfo>,
    
) {
    if !bmw.start {
        return;
    }
    // println!("Started---");

    let time_since_start = time.elapsed_secs() - bmw.started_at;
    let Some(next) = osu.hit_objects.get(bmw.next) else {
        bmw.start = false;
        info!("beatmap ended");
        return;
    };
    if time_since_start < (next.time - osu.real_approach_time) {
        return;
    }
    // println!("current : {}", bmw.next);
    bmw.next += 1;
    // println!("Doing smth");

    oho_msg.write(next.clone());
}
