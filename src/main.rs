use std::sync::Mutex;

use bevy::prelude::*;
use bevy_enoki::prelude::Rval;
use bevy_enoki::{EnokiPlugin, Particle2dEffect};
use bevy_vello::vello::peniko::color::Srgb;

mod beatmaps;
mod circles;
mod game_debug;
mod osuparser;
mod public_resources;

use crate::circles::etc::*;
use crate::osuparser::OsuHitObject;
use crate::public_resources::*;

const CIRCLE_VISUAL_MULTIPLIER: f32 = 0.8;

pub const WORLD_BG: RenderLayers = RenderLayers::layer(0);
pub const WORLD_FG: RenderLayers = RenderLayers::layer(1);
// pub const WORLD_FG_EXTRA: RenderLayers = RenderLayers::layer(2);

pub const SVG_MODE: bool = true;


static BEATMAP_PATH: Mutex<&str> = Mutex::new("assets/beatmaps/hikarunara_hard.osu");
static MUSIC_PATH: Mutex<&str> = Mutex::new("beatmaps/hikarunara.mp3");


fn setup_world(
    assets: Res<AssetServer>,
    mut particles: ResMut<Assets<Particle2dEffect>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hitobj_writer: MessageWriter<OsuHitObject>,
    window: Single<(&Window, Entity)>,
    mut load_bmap_msg: MessageWriter<LoadBeatmap>,
    mut general_info: ResMut<GeneralInfo>,
) {
    commands.spawn((
        Camera2d::default(), // replaces Camera2dBundle
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
    // commands.spawn((
    //     Camera2d::default(),
    //     Camera {
    //         order: 2,
    //         clear_color: ClearColorConfig::None,
    //         ..default()
    //     },
    //     WORLD_FG_EXTRA,
    //     Cameraz2,
    // ));

    // commands.spawn((Camera2d::default(), Transform::from_xyz(0.0, 0.0, 1000.0), VelloView));

    // let sprite = Sprite::from_color();

    general_info.real_circle_radius = 49.92 * (window.0.size().y / 480.0);

    let circle = Circle::new(49.92 * (window.0.size().y / 480.0));
    let circle_handle = meshes.add(circle);
    let circle_mesh = Mesh2d(circle_handle);

    let circle_ring = circle.to_ring(6.0 * window.0.size().y / 1080.0);
    let circle_ring_handle = meshes.add(circle_ring);
    let circle_ring_mesh = Mesh2d(circle_ring_handle);

    let mred = MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0)));
    let mwhite = MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0)));

    let circle_asset: Handle<Image> = assets.load("skins/circle.png");
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

    let main_svg = assets.load("skins/circle.svg");
    let slider_svg = assets.load("skins/circle_slider.svg");

    // let mut great_hit = assets.load("skins/particles/great.ron");

    // let great_hit_inner = particles.get_mut(great_hit.id()).unwrap();
    // great_hit_inner.linear_speed.as_mut().unwrap().0 = 10.0;

    commands.insert_resource(GlobalParticleEffects {
        great_hit: assets.load("skins/particles/great.ron"),
        ok_hit: assets.load("skins/particles/ok.ron"),
        meh_hit: assets.load("skins/particles/meh.ron"),
        miss: assets.load("skins/particles/miss.ron"),
        tick_hit: assets.load("skins/particles/tick_hit.ron"),
        tick_miss: assets.load("skins/particles/tick_miss.ron"),
        tick_ok: assets.load("skins/particles/tick_ok.ron"),
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

    let p = Point { x: 0, y: 0 };

    let pos = p.to_real_pos(window.0.size());
    println!("{:?}", pos);

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
        audio: MUSIC_PATH.lock().unwrap().to_owned()
    });

    commands
        .spawn(
            (Node {
                width: percent(100),
                height: percent(5),
                top: percent(5),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::Start,
                ..Default::default()
            }),
        )
        .with_child((
            ScoreGui,
            Text::new("Score: "),
            TextFont::from_font_size(50.0),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));
    commands
        .spawn(
            (Node {
                width: percent(100),
                height: percent(5),
                top: percent(90),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::End,
                ..Default::default()
            }),
        )
        .with_child((
            AccuracyGui,
            Text::new("Accuracy: "),
            TextFont::from_font_size(50.0),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ));

    commands
        .spawn(
            (Node {
                width: percent(100),
                height: percent(5),
                top: percent(85),
                left: percent(2),
                justify_content: JustifyContent::Start,
                align_content: AlignContent::End,
                ..Default::default()
            }),
        )
        .with_child((
            ComboGui,
            Text::new("Combo: "),
            TextFont::from_font_size(40.0),
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
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
                handle: assets.load("skins/helpers/crosshair.png"),

                texture_atlas: None,
                flip_x: false,
                flip_y: false,

                rect: None,
                hotspot: (0, 0),
            }),
        ),));
}



use crate::osuparser::{OsuBeatmap, Point};
use bevy::camera::visibility::RenderLayers;
use bevy_vello::VelloPlugin;
use bevy_vello::render::VelloView;

mod mouse_pos_system;

fn start_game() {
    // osuparser::parse_osu_file(Path::new("bad_apple.osu")).unwrap();
    // return;

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // resolution:
                    //     bevy_window::WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                    resolution: bevy_window::WindowResolution::new(1400, 720)
                        .with_scale_factor_override(1.0),
                    mode: bevy_window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    // mode: bevy_window::WindowMode::Windowed,
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
    app.add_message::<StartMovingSlider>();
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

    // app.add_systems(FixedUpdate, circles::clicking::circle_click);

    app.run();
}



fn main() {
    start_game();
    

}