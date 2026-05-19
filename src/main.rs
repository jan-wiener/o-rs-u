use std::path::Path;
use std::sync::Mutex;
use std::path::PathBuf;
use std::sync::LazyLock;

use clap::Parser;

use bevy::asset::io::AssetSourceId;
use bevy::asset::{AssetPath, embedded_asset};
use bevy::prelude::*;

use bevy_enoki::{EnokiPlugin, Particle2dEffect};



use crate::osuparser::{OsuBeatmap};
use bevy::camera::visibility::RenderLayers;
use bevy_vello::VelloPlugin;
use bevy_vello::render::VelloView;

use crate::circles::etc::*;
use crate::osuparser::osutypes::*;
use crate::public_resources::*;

mod mouse_pos_system;
mod beatmaps;
mod circles;
mod game_debug;
mod osuparser;
mod public_resources;
mod cli;





const CIRCLE_VISUAL_MULTIPLIER: f32 = 0.8;

pub const WORLD_BG: RenderLayers = RenderLayers::layer(0);
pub const WORLD_FG: RenderLayers = RenderLayers::layer(1);

pub const SVG_MODE: bool = true;




pub const CRATE_NAME: &str = "o_rs_u";


static BEATMAP_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| {Mutex::new("assets/beatmaps/hikarunara_hard.osu".to_string())});
// static MUSIC_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| {Mutex::new("beatmaps/hikarunara.mp3".to_string())});
static IMAGE_PATH: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| {Mutex::new(None)});



// Get Embedded path
pub fn gep(path: &str) -> AssetPath<'_> {
    let path = String::from("assets/") + path;
    let path = PathBuf::from(Path::new(CRATE_NAME).join(path));
    let source = AssetSourceId::from("embedded");
    let asset_path = AssetPath::from_path_buf(path).with_source(source);
    return asset_path;

}



fn setup_world(
    assets: Res<AssetServer>,
    _particles: ResMut<Assets<Particle2dEffect>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _hitobj_writer: MessageWriter<OsuHitObject>,
    window: Single<(&Window, Entity)>,
    mut load_bmap_msg: MessageWriter<LoadBeatmap>,
    mut general_info: ResMut<GeneralInfo>,
) {
    commands.spawn((
        Camera2d::default(), 
        Camera {
            order: 0,
            ..default()
        },
        WORLD_BG,
        Cameraz0,
    ));

    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        WORLD_FG,
        VelloView,
    ));


    general_info.real_circle_radius = 49.92 * (window.0.size().y / 480.0);

    let circle = Circle::new(49.92 * (window.0.size().y / 480.0));
    let circle_handle = meshes.add(circle);
    let circle_mesh = Mesh2d(circle_handle);

    let circle_ring = circle.to_ring(6.0 * window.0.size().y / 1080.0);
    let circle_ring_handle = meshes.add(circle_ring);
    let circle_ring_mesh = Mesh2d(circle_ring_handle);

    let _mred = MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0)));
    let mwhite = MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0)));

    let _circle_asset: Handle<Image> = assets.load(gep("skins/circle.png"));
    let mut m = ColorMaterial::default();
    // m.texture = Some(circle_asset);

    let alpha = 1.0;
    m.color = Color::srgba(1.0, 1.0, 1.0, alpha);

    let main_mat = MeshMaterial2d(materials.add(m.clone()));

    m.color = Color::srgba(1.0, 0.65, 0.0, alpha);
    let meh_mat = MeshMaterial2d(materials.add(m.clone()));

    m.color = Color::srgba(1.0, 1.0, 0.0, alpha);
    let ok_mat = MeshMaterial2d(materials.add(m.clone()));

    m.color = Color::srgba(0.6, 1.0, 0.0, alpha);
    let great_mat = MeshMaterial2d(materials.add(m));

    // let main_svg = assets.load("skins/circle.svg");
    let main_svg = assets.load(gep("skins/circle.svg"));
    let slider_svg = assets.load(gep("skins/circle_slider.svg"));

    // let mut great_hit = assets.load("skins/particles/great.ron");

    // let great_hit_inner = particles.get_mut(great_hit.id()).unwrap();
    // great_hit_inner.linear_speed.as_mut().unwrap().0 = 10.0;

    commands.insert_resource(GlobalParticleEffects {
        great_hit: assets.load(gep("skins/particles/great.ron")),
        ok_hit: assets.load(gep("skins/particles/ok.ron")),
        meh_hit: assets.load(gep("skins/particles/meh.ron")),
        miss: assets.load(gep("skins/particles/miss.ron")),
        tick_hit: assets.load(gep("skins/particles/tick_hit.ron")),
        tick_miss: assets.load(gep("skins/particles/tick_miss.ron")),
        tick_ok: assets.load(gep("skins/particles/tick_ok.ron")),
        done_scaling: false,
    });

    commands.insert_resource(CircleMaterials {
        meh_mat,
        ok_mat,
        great_mat,
        main: circle_mesh,
        main_mat,
        ring: circle_ring_mesh,
        ring_mat: mwhite,
        main_svg,
        slider_svg,
    });

    // let p = Point { x: 0, y: 0 };

    // let pos = p.to_real_pos(window.0.size());
    // println!("{:?}", pos);

    let mut o = OsuHitObject::default();
    o.trpos = Some(Vec2::new(100.0, 100.0));
    // hitobj_writer.write(o);

    let s = Sprite::from_color(
        Color::srgba(1.0, 0.0, 0.0, 0.1),
        Vec2::new(512.0, 384.0) * (window.0.height() / 480.0),
    );

    commands.spawn((s, Transform::from_xyz(0.0, 0.0, 0.0)));

    let default_audio_source = assets.add(AudioSource {
        bytes: std::sync::Arc::new([]),
    });

    commands.spawn((
        GameAudio,
        AudioPlayer::new(default_audio_source),
        PlaybackSettings::ONCE.paused(),
    ));

    load_bmap_msg.write(LoadBeatmap {
        path: BEATMAP_PATH.lock().unwrap().to_owned(),
        audio_override: None,  //MUSIC_PATH.lock().unwrap().to_owned()
    });



    let default_text_shadow = TextShadow{offset: Vec2::new(4.0,-4.0), color: Color::srgb(1.0, 0.0, 0.0)};
    let default_text_size = (window.0.size().y / 1080.0) * 50.0;
    let big_font = TextFont::from_font_size(default_text_size);
    let small_font = TextFont::from_font_size(default_text_size * 0.8);

    commands
        .spawn(
            Node {
                width: percent(100),
                height: percent(5),
                top: percent(5),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::Start,
                ..Default::default()
            } ,
        )
        .with_child((
            ScoreGui,
            Text::new("Score: "),
            big_font.clone(),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            default_text_shadow,
        ));
    commands
        .spawn(
            Node {
                width: percent(100),
                height: percent(5),
                top: percent(90),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::End,
                ..Default::default()
            } ,
        )
        .with_child((
            AccuracyGui,
            Text::new("Accuracy: "),
            big_font,
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            default_text_shadow,
        ));

    commands
        .spawn(
            Node {
                width: percent(100),
                height: percent(5),
                top: percent(85),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::End,
                ..Default::default()
            } ,
        )
        .with_child((
            ComboGui,
            Text::new("Combo: "),
            small_font,
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
            default_text_shadow,
        ));

    let spr = Sprite::from_color(Color::srgb(0.0, 1.0, 0.0), Vec2::new(20.0, 20.0));
    commands.spawn((
        spr,
        Visibility::Hidden,
        Transform::from_xyz(0.0, 0.0, 960.0),
        WhatShouldIClick,
        WORLD_FG,
    ));

    commands
        .entity(window.1)
        .insert((bevy_window::CursorIcon::Custom(
            bevy_window::CustomCursor::Image(bevy_window::CustomCursorImage {
                handle: assets.load(gep("skins/helpers/crosshair.png")),

                texture_atlas: None,
                flip_x: false,
                flip_y: false,

                rect: None,
                hotspot: (0, 0),
            }),
        ),));


        let mut spr = Sprite::default();
        println!("Color: {:?},", spr.color);
        spr.color = Color::srgba(0.7, 0.7, 0.7, 1.0);
        println!("Color: {:?},\n\n", spr.color);
        commands.spawn((
            Transform::default(),
            spr,
            GameBackground
        ));
}





fn start_game() {
    // osuparser::parse_osu_file(Path::new("bad_apple.osu")).unwrap();

    

    let mut app = App::new();

    

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution:
                        bevy_window::WindowResolution::new(800, 600).with_scale_factor_override(1.0),
                    mode: bevy_window::WindowMode::Windowed,

                    // resolution: bevy_window::WindowResolution::new(1400, 720)
                        // .with_scale_factor_override(1.0),
                    // mode: bevy_window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                   
                    present_mode: bevy_window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(ImagePlugin {
                default_sampler: bevy::image::ImageSamplerDescriptor {
                    mag_filter: bevy::image::ImageFilterMode::Linear,
                    min_filter: bevy::image::ImageFilterMode::Linear,
                    ..Default::default()
                },
            }),
    );


    app.add_plugins(mouse_pos_system::MousePosPlugin);
    app.add_plugins(EnokiPlugin);

    app.add_plugins(game_debug::GameDebugPlugin);

    let mut vello = VelloPlugin::default();
    vello.canvas_render_layers = WORLD_FG;
    app.add_plugins(vello);

    app.insert_resource(Time::<Fixed>::from_hz(240.0));
    // app.insert_resource(Time::<Virtual>::);

    app.add_message::<OsuHitObject>();
    app.add_message::<RemoveCircle>();
    app.add_message::<LoadBeatmap>();
    app.add_message::<DrawLine>();
    // app.add_message::<StartMovingSlider>();
    app.add_message::<AddScore>();
    app.add_message::<TickCheck>();
    app.add_message::<DrawTick>();

    app.init_gizmo_group::<LineGizmos>();

    app.insert_resource(MouseInfo::default());
    app.insert_resource(OsuBeatmap::default());
    app.insert_resource(BeatmapWorkerInfo::default());
    app.insert_resource(MovingSlidersRes::default());
    app.insert_resource(GeneralInfo::default());
    app.insert_resource(ScoreInfo::default());

    app.add_systems(Startup, setup_world);
    app.add_systems(
        Update,
        (
            circles::scoring::scale_particles_once,
            circles::what_should_i_click,
            circles::pausing::pausing_system.before(beatmaps::play_audio),
            circles::rings::shrink_ring,
            circles::summon_circle,
            circles::clicking::circle_click.before(circles::sliders::move_slider),
            beatmaps::load_osu_beatmap,
            beatmaps::beatmap_worker,
            circles::sliders::draw_from_points,
            circles::sliders::remove_line,
            circles::sliders::move_slider.before(circles::clicking::remove_circle),
            circles::clicking::remove_circle,
            circles::scoring::score_system,
            circles::change_material_system,
            circles::sliders::tick_check,
            circles::sliders::draw_tick,
            circles::sliders::remove_tick,
            beatmaps::play_audio,
        ),
    );


    embedded_asset!(app, "assets/skins/circle.svg");
    embedded_asset!(app, "assets/skins/circle.png");
    embedded_asset!(app, "assets/skins/circle_slider.svg");
    embedded_asset!(app, "assets/skins/circle_slider.svg");
    embedded_asset!(app, "assets/skins/particles/great.ron");
    embedded_asset!(app, "assets/skins/particles/meh.ron");
    embedded_asset!(app, "assets/skins/particles/miss.ron");
    embedded_asset!(app, "assets/skins/particles/ok.ron");
    embedded_asset!(app, "assets/skins/particles/tick_hit.ron");
    embedded_asset!(app, "assets/skins/particles/tick_miss.ron");
    embedded_asset!(app, "assets/skins/particles/tick_ok.ron");
    embedded_asset!(app, "assets/skins/helpers/crosshair.png");







    // println!("{:?}",embedded_path!("assets/skins/circle.svg"));

    // app.add_systems(FixedUpdate, circles::clicking::circle_click);

    app.run();
}



fn main() {
    let mut cli = cli::Cli::from_args(cli::Args::parse());
    if cli.uses_cli {
        cli.extract_osz_file().unwrap();
    }
    
    // println!("args: {:?}", args);
    


    // let path_unzip = osuparser::unzipper::unzip_osufile("./440169 Goose house - Hikaru nara.osz", "beatmap_extract/440169 Goose house - Hikaru nara.osz").unwrap();
    // println!("{}", path_unzip);
    // osuparser::unzipper::get_osu_files_from_extracted_osz_file(&path_unzip).unwrap();
    // return;


    start_game();
    

}