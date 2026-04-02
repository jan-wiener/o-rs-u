use bevy::prelude::*;

mod beatmaps;
mod circles;
mod osuparser;
mod public_resources;

use crate::circles::etc::*;
use crate::osuparser::OsuHitObject;
use crate::public_resources::*;

const CIRCLE_VISUAL_MULTIPLIER: f32 = 0.8;

fn setup_world(
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hitobj_writer: MessageWriter<OsuHitObject>,
    window: Single<&Window>,
    mut load_bmap_msg: MessageWriter<LoadBeatmap>,
    mut general_info: ResMut<GeneralInfo>,
) {
    commands.spawn((Camera2d::default(), Transform::from_xyz(0.0, 0.0, 1000.0)));

    // let sprite = Sprite::from_color();

    general_info.real_circle_radius = 49.92 * (window.size().y / 480.0);

    let circle = Circle::new(49.92 * (window.size().y / 480.0));
    let circle_handle = meshes.add(circle);
    let circle_mesh = Mesh2d(circle_handle);

    let circle_ring = circle.to_ring(2.0);
    let circle_ring_handle = meshes.add(circle_ring);
    let circle_ring_mesh = Mesh2d(circle_ring_handle);

    let mred = MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0)));
    let mwhite = MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0)));

    let circle_asset = assets.load("circle.png");
    let mut m = ColorMaterial::default();
    m.texture = Some(circle_asset);

    let main_mat = MeshMaterial2d(materials.add(ColorMaterial::default()));
    
    let meh_mat = MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.65, 0.0))));
    let ok_mat = MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(1.0, 1.0, 0.0))));
    let great_mat = MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.6, 1.0, 0.0))));



    commands.insert_resource(CircleMaterials {
        meh_mat,
        ok_mat,
        great_mat,
        main: circle_mesh,
        main_mat,
        ring: circle_ring_mesh,
        ring_mat: mwhite,
    });

    let p = Point { x: 0, y: 0 };

    let pos = p.to_real_pos(window.size());
    println!("{:?}", pos);

    let mut o = OsuHitObject::default();
    o.trpos = Some(Vec2::new(100.0, 100.0));
    // hitobj_writer.write(o);

    let s = Sprite::from_color(
        Color::srgba(1.0, 0.0, 0.0, 0.1),
        Vec2::new(512.0, 384.0) * (window.height() / 480.0),
    );

    commands.spawn((s, Transform::from_xyz(0.0, 0.0, 0.0)));

    load_bmap_msg.write(LoadBeatmap {
        path: "o.osu".into(),
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



    // commands.spawn((
    //     // Transform::from_xyz(0.0,0.0,10.0),
    //     ScoreGui,
    //     Text::new("Test"),
    //     TextColor(Color::srgb(1.0, 1.0, 1.0)),
    //     // TextLayout::new_with_justify(Justify::Center).with_no_wrap(),
    // ));
}

// fn mouseclick_to_circle_summon(
//     time: Res<Time>,
//     mut mouse_info: ResMut<MouseInfo>,
//     mouse_buttons: Res<ButtonInput<MouseButton>>,
//     mut ringwriter: MessageWriter<SummonCircle>,
// ) {
//     if !mouse_buttons.just_pressed(MouseButton::Left) {
//         return;
//     }
//     let mut rng = rand::rng();
//     ringwriter.write(SummonCircle {
//         size: 50.0,
//         pos: mouse_info.pos,
//         ring: Ring{original_scale: 1.7, time_to_shrink: rng.random::<f32>()},
//     });
// }

use crate::osuparser::{OsuBeatmap, Point};

mod mouse_pos_system;

fn main() {
    // osuparser::parse_osu_file(Path::new("bad_apple.osu")).unwrap();
    // return;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution:
                bevy_window::WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
            ..Default::default()
        }),
        ..Default::default()
    }));

    app.add_plugins(mouse_pos_system::MousePosPlugin);

    app.insert_resource(Time::<Fixed>::from_hz(240.0));

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
            circles::rings::shrink_ring,
            circles::summon_circle,
            circles::clicking::circle_click,
            
            beatmaps::load_osu_beatmap,
            beatmaps::beatmap_worker,
            circles::sliders::draw_from_points,
            circles::sliders::remove_line,
            (circles::sliders::move_slider, circles::clicking::remove_circle).chain(),
            circles::scoring::score_system,
            circles::change_material_system,
            circles::sliders::tick_check,

            circles::sliders::draw_tick,
            circles::sliders::remove_tick,

        ),
    );

    // app.add_systems(FixedUpdate, circles::clicking::circle_click);

    app.run();
}
