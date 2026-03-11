use std::path::Path;

use bevy::ecs::message;
use bevy::ecs::system::entity_command::despawn;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use rand::RngExt;

mod osuparser;

#[derive(Component)]
struct Ring {
    original_scale: f32,
    slider_mode: bool, // time_to_shrink: f32,
}
impl Ring {
    fn new(original_scale: f32) -> Self {
        Self {
            original_scale,
            slider_mode: false,
            // time_to_shrink,
        }
    }
}
impl Default for Ring {
    fn default() -> Self {
        Self {
            original_scale: 1.5,
            slider_mode: false,
            // time_to_shrink: 0.5,
        }
    }
}

#[derive(Component)]
struct CircleInfo {
    clicked: bool,
    circle_type: OsuHitObjectType,

    points: Option<Vec<Vec2>>,
    segments: Option<usize>,
    slides: u32,

    original_pos: Vec2,

    size: f32,
}

impl Default for CircleInfo {
    fn default() -> Self {
        Self {
            clicked: false,
            circle_type: OsuHitObjectType::Circle(false),
            points: None,
            size: 0.0,
            segments: None,
            original_pos: Vec2::default(),
            slides: 1,
        }
    }
}

// #[derive(Message)]
// struct SummonCircle {
//     size: f32,
//     pos: Vec2,
//     ring: Ring,
// }

#[derive(Message)]
struct RemoveCircle {
    entity: Entity,
}

fn remove_circle(mut circles_to_remove: ResMut<Messages<RemoveCircle>>, mut commands: Commands) {
    for remove_circle_msg in circles_to_remove.drain() {
        commands
            .entity(remove_circle_msg.entity)
            .queue_silenced(despawn());
        // commands.entity(remove_circle_msg.entity).despawn();
    }
}

#[derive(Resource)]
struct CircleMaterials {
    main: Mesh2d,
    main_mat: MeshMaterial2d<ColorMaterial>,
    ring: Mesh2d,
    ring_mat: MeshMaterial2d<ColorMaterial>,
}

use crate::osuparser::{CurveType, OsuHitObject, OsuHitObjectType};

fn summon_circle(
    circlemats: Res<CircleMaterials>,
    osu: Res<OsuBeatmap>,
    window: Single<&Window>,
    mut circles_to_summon: ResMut<Messages<OsuHitObject>>,
    mut drawline_writer: MessageWriter<DrawLine>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for osuhitobj in circles_to_summon.drain() {
        // println!("OsuHitObject found");

        let pos = osuhitobj.trpos.expect("somebody didnt convert pos");

        if !osuhitobj.hitobjecttype.is_circle_like() {
            continue;
        }

        // let size = osu.get_real_circle_size(window.size());
        let mut points: Option<Vec<Vec2>> = None;
        let mut segments: Option<usize> = None;
        let mut slides: u32 = 1;

        match osuhitobj.hitobjecttype {
            OsuHitObjectType::Slider(_) => {
                let slider_info = osuhitobj.slider_params.unwrap();

                slides = slider_info.slides;
                // let tr_curve_points_inner = slider_info.trcurve_points;
                let mut points_inner: Vec<Vec2> = vec![];

                // let iter_steps = (0..10).map(|x| x as f32 / 10.0);
                let mut length = 0.0;
                println!("t: {:?}", slider_info.curve_type);
                match slider_info.curve_type {
                    CurveType::Bezier => {
                        let mut splits: Vec<Vec<Vec2>> = vec![vec![]];
                        for (idx, trpoint) in slider_info.trcurve_points.iter().enumerate() {
                            if idx.checked_sub(1).is_some() && let Some(last) = slider_info.trcurve_points.get(idx - 1) {
                                if last == trpoint {
                                    splits.push(vec![]);
                                }
                            }
                            splits.last_mut().unwrap().push(trpoint.clone());
                        }

                        for split in &splits {
                            length += bezier_length(split, 10);
                        }
                        // length = bezier_length(&slider_info.trcurve_points, 10);

                        // println!("splits: {}", splits.len());

                        let steps = (length / 10.0 / splits.len() as f32) as i32 ;
                        segments = Some(splits.len());
                        for split in splits {
                            for step in (0..steps).map(|x| x as f32 / steps as f32) {
                                points_inner
                                    .push(bezier_casteljau(&split, step));
                            }
                        }
                        length = vec_vec2_len(&points_inner);
                        
                        // println!("len: {}\nPoints: {:?}", length, points_inner);
                    }
                    CurveType::Centripetal => {
                        let spline =
                            CentripetalCatmullRomSpline2d::new(&slider_info.trcurve_points);
                        length = spline.length(10 / spline.segments.len());
                        let steps = (length / 10.0) as i32;
                        for step in (0..steps).map(|x| x as f32 / steps as f32) {
                            points_inner.push(bezier_casteljau(
                                &slider_info.trcurve_points,
                                step * (spline.segments.len() as f32),
                            ));
                        }
                    },
                    CurveType::Linear => {

                        let mut key_points = slider_info.trcurve_points.clone();
                        
                        length = vec_vec2_len(&key_points);
                        let steps = length / 10.0;
                        println!("{:?}", key_points);
                        for (idx, key_point) in key_points.iter().enumerate() {
                            if idx.checked_sub(1).is_some() && let Some(last) = key_points.get(idx-1) {
                                let seg_length = key_point.distance(*last);
                                let seg_steps = seg_length / length * steps;
                                let seg_step_length = seg_length / seg_steps;

                                println!("{}, {}, {}", seg_length, seg_steps, seg_step_length);

                                let mut seg_length_covered = 0.0;
                                let mut next = last.clone();
                                loop {
                                    let step = (key_point-last).normalize() * seg_step_length;
                                    next = next + step;
                                    seg_length_covered += step.length();
                                    if seg_length_covered > seg_length{
                                        println!("broken");
                                        break;
                                    }
                                    points_inner.push(next);

                                }
                            }
                        }
                        // println!("{:?}", points_inner);



                    }
                    _ => {warn!("Curve for a slider not loaded!")}
                }
                // println!("{}", osu.get_time_to_complete_slider(length));
                drawline_writer.write(DrawLine {
                    points: points_inner.clone(),
                    lifetime: osu.get_time_to_complete_slider(length) * slides as f32 + osu.real_approach_time ,
                });
                points = Some(points_inner);
            }
            _ => {}
        }

        

        let mut centcmds = commands.spawn((
            circlemats.main.clone(),
            circlemats.main_mat.clone(),
            Transform::from_translation(pos.extend(1.0)),
            CircleInfo {
                original_pos: pos,
                clicked: false,
                size: osu.real_circle_size,
                circle_type: osuhitobj.hitobjecttype,
                points,
                segments,
                slides,
            },
        ));

        // let mut tr = Transform::from_translation(ring.pos.extend(1.0));
        let mut tr = Transform::from_translation(Vec3::splat(0.0));
        tr.scale = Vec3::splat(2.0);

        let ring = Ring::new(2.0);

        centcmds.with_child((
            circlemats.ring.clone(),
            circlemats.ring_mat.clone(),
            tr,
            ring,
        ));
    }
}

fn setup_world(
    assets: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hitobj_writer: MessageWriter<OsuHitObject>,
    window: Single<&Window>,
    mut load_bmap_msg: MessageWriter<LoadBeatmap>,
) {
    commands.spawn((Camera2d::default(), Transform::from_xyz(0.0, 0.0, 1000.0)));

    // let sprite = Sprite::from_color();

    let circle = Circle::new(50.0);
    let circle_handle = meshes.add(circle);
    let circle_mesh = Mesh2d(circle_handle);

    let circle_ring = circle.to_ring(2.0);
    let circle_ring_handle = meshes.add(circle_ring);
    let circle_ring_mesh = Mesh2d(circle_ring_handle);

    let mred = MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0)));
    let mwhite = MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0)));

    // let circle_asset = assets.load("circle.png");
    let mut m = ColorMaterial::default();
    // m.texture = Some(circle_asset);
    let mcircle = MeshMaterial2d(materials.add(m));

    commands.insert_resource(CircleMaterials {
        main: circle_mesh,
        main_mat: mcircle,
        ring: circle_ring_mesh,
        ring_mat: mwhite,
    });

    let p = Point { x: 0, y: 0 };

    let pos = p.to_real_pos(window.size());
    println!("{:?}", pos);

    let mut o = OsuHitObject::default();
    o.trpos = Some(Vec2::new(100.0, 100.0));
    hitobj_writer.write(o);

    let s = Sprite::from_color(
        Color::srgba(1.0, 0.0, 0.0, 0.1),
        Vec2::new(512.0, 384.0) * (window.height() / 480.0),
    );

    commands.spawn((s, Transform::from_xyz(0.0, 0.0, 0.0)));

    load_bmap_msg.write(LoadBeatmap {
        path: "bad_apple.osu".into(),
    });
}

fn circle_click(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    kb: Res<ButtonInput<KeyCode>>,

    mouse_info: Res<MouseInfo>,
    mut circles_q: Query<(&Transform, &mut CircleInfo, Entity, &Children)>,
    mut ring_q: Query<(&mut Transform, &mut Ring, &ChildOf), Without<CircleInfo>>,
    mut commands: Commands,
    mut removewriter: MessageWriter<RemoveCircle>,
    mut slider_res: ResMut<MovingSlidersRes>,
) {
    if !mouse_button.just_pressed(MouseButton::Left)
        && !kb.just_pressed(KeyCode::KeyX)
        && !kb.just_pressed(KeyCode::KeyY)
    {
        return;
    }

    let mut selected_ent: (Option<Entity>, f32) = (None, -100.0);

    for (tr, mut circleinfo, centity, children) in &mut circles_q {
        if mouse_info.pos.distance(tr.translation.truncate()) <= circleinfo.size
            && selected_ent.1 < tr.translation.z
            && !circleinfo.clicked
        {
            selected_ent = (Some(centity), tr.translation.z);
        }
    }

    if let Some(ent) = selected_ent.0 {
        let (tr, mut circleinfo, entity, children) = circles_q.get_mut(ent).unwrap();
        circleinfo.clicked = true;
        match circleinfo.circle_type {
            // println!("___");
            OsuHitObjectType::Circle(_) => {
                removewriter.write(RemoveCircle { entity: ent });
            }
            OsuHitObjectType::Slider(_) => {
                slider_res.sliders.push(MovingSlider {
                    entity,
                    started_at: time.elapsed_secs(),
                    // target_slides: circleinfo.slides,
                    done_slides: 0,
                    
                });
                ring_q
                    .get_mut(children.first().unwrap().to_owned())
                    .unwrap()
                    .1
                    .slider_mode = true;
            }
            _ => {}
        }
    }
}

fn shrink_ring(
    time: Res<Time>,
    ring_q: Query<(&mut Transform, &mut Ring, &ChildOf)>,
    mut circle_q: Query<&mut CircleInfo>,
    mut rmcrc: MessageWriter<RemoveCircle>,
    osu: Res<OsuBeatmap>,
    mut slider_res: ResMut<MovingSlidersRes>,
) {
    if time.elapsed_secs() < 1.5 {
        return;
    }

    for (mut tr, mut ring, ch) in ring_q {
        if tr.scale.x > 1.0 {
            // println!("Shrunk @ {}", time.elapsed_secs());
            tr.scale -= Vec3::splat(1.0) * time.delta_secs() / osu.real_approach_time;
        } else if !ring.slider_mode {
            let mut circle = circle_q.get_mut(ch.parent()).unwrap();
            circle.clicked = true;
            match circle.circle_type {
                OsuHitObjectType::Circle(_) => {
                    rmcrc.write(RemoveCircle {
                        entity: ch.parent(),
                    });
                }
                OsuHitObjectType::Slider(_) => {
                    slider_res.sliders.push(MovingSlider {
                        entity: ch.parent(),
                        started_at: time.elapsed_secs(),
                        // target_slides: circle.slides,
                        done_slides: 0
                    });
                    ring.slider_mode = true;
                }
                _ => {}
            }
        }
    }
}

#[derive(Message)]
struct StartMovingSlider {
    entity: Entity,
}

struct MovingSlider {
    entity: Entity,
    started_at: f32,
    done_slides: u32,
    // target_slides: u32
}
#[derive(Resource, Default)]
struct MovingSlidersRes {
    sliders: Vec<MovingSlider>,
}

fn vec_vec2_len(vec2s: &[Vec2]) -> f32 {
    let mut total = 0.0;
    let mut last: &Vec2;
    for i in 1..vec2s.len() {
        total += vec2s[i - 1].distance(vec2s[i]);
    }
    total
}

fn move_slider(
    mut circles: Query<(&CircleInfo, &mut Transform)>,
    mut slider_res: ResMut<MovingSlidersRes>,
    time: Res<Time>,
    osu: Res<OsuBeatmap>,
    mut remove_circ: MessageWriter<RemoveCircle>,
    mut commands: Commands,
) {
    let mut to_remove = vec![];

    for (idx, movingslider) in slider_res.sliders.iter().enumerate() {
        if commands.get_entity(movingslider.entity).is_err() {
            to_remove.push(idx);
            continue;
        }

        let (circleinfo, mut tr) = circles.get_mut(movingslider.entity).unwrap();
        let points_inner = circleinfo.points.as_ref().unwrap();
        let length = vec_vec2_len(points_inner);

        // println!("ttc: {}\ndelta_slider: {}", osu.get_time_to_complete_slider(length), (time.elapsed_secs() - movingslider.started_at));
        let mut t = (time.elapsed_secs() - movingslider.started_at)
            / osu.get_time_to_complete_slider(length);
        
        if t > (circleinfo.slides as f32) {
            remove_circ.write(RemoveCircle {
                entity: movingslider.entity,
            });
            continue;
        }
        t = if t % 2.0 > 1.0 {
            2.0 - (t % 2.0)
        } else {
            t % 2.0
        };


        let mut new_pos: Option<Vec2> = None;
        let target_distance = length * t;
        let mut total = 0.0;
        let mut last: Vec2 = circleinfo.original_pos;
        for v in points_inner {
            total += last.distance(v.clone());
            last = v.clone();
            if total >= target_distance {
                new_pos = Some(v.clone());
                break;
            }
        }
        let Some(new_pos) = new_pos else {
            remove_circ.write(RemoveCircle {
                entity: movingslider.entity,
            });
            continue;
        };

        // let new_pos_idx = (points_inner.len() as f32 * t) as usize;
        // let new_pos: Vec2 = points_inner[new_pos_idx];

        tr.translation = new_pos.extend(tr.translation.z);
        // println!("{:?}", new_pos);
    }

    let mut offset = 0;
    for i in to_remove {
        slider_res.sliders.remove(i - offset);
        offset += 1;
    }
}

#[derive(Resource, Default)]
struct MouseInfo {
    pos: Vec2,
    velocity: Vec2,
    on_screen: bool,
    pressed: bool,
}

fn mouse_position_system(
    time: Res<Time>,
    mut mouse_info: ResMut<MouseInfo>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    // mut posgui: Single<&mut Text, With<PosGUI>>,
    camera_s: Single<&Transform, (With<Camera>)>,
) {
    if let Some(cursor_pos) = window.cursor_position() {
        mouse_info.on_screen = true;

        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
        let camera_tr = *camera_s;
        let camera_pos = &camera_tr.translation;
        let rel_pos = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
        let in_game_middle = window_size / 2.0;

        let mut in_game_pos = in_game_middle * rel_pos - Vec2::new(camera_pos.x, camera_pos.y);
        in_game_pos.y *= -1.0;

        let velocity = (in_game_pos - mouse_info.pos) / time.delta_secs();
        mouse_info.pos = in_game_pos;

        mouse_info.velocity = velocity;

        if !mouse_buttons.pressed(MouseButton::Left) {
            mouse_info.pressed = false;
        } else {
            mouse_info.pressed = true;
        }
    } else {
        mouse_info.pressed = false;
        mouse_info.on_screen = false;
    }
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

mod catmull;
use crate::catmull::*;

fn bezier(points: &[Vec2], t: f32) -> Vec2 {
    let n = points.len() - 1;
    let mut result = Vec2::ZERO;

    for (i, p) in points.iter().enumerate() {
        let bin = binomial(n, i) as f32;
        let term = bin * (1.0 - t).powi((n - i) as i32) * t.powi(i as i32);

        result += *p * term;
    }

    result
}
fn bezier_length_normal(points: &[Vec2], steps: usize) -> f32 {
    let mut total = 0.0;
    let mut prev = bezier(points, 0.0);

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let p = bezier(points, t);
        total += prev.distance(p);
        prev = p;
    }

    total
}

fn bezier_casteljau(points: &[Vec2], t: f32) -> Vec2 {
    let mut pts = points.to_vec();
    let n = pts.len();
    for k in 1..n {
        for i in 0..(n - k) {
            pts[i] = pts[i].lerp(pts[i + 1], t);
        }
    }
    pts[0]
}
fn binomial(n: usize, k: usize) -> usize {
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}
fn bezier_length(points: &[Vec2], steps: usize) -> f32 {
    let mut total = 0.0;
    let mut prev = bezier_casteljau(points, 0.0);

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let p = bezier_casteljau(points, t);
        total += prev.distance(p);
        prev = p;
    }

    total
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct LineGizmos {}

#[derive(Message)]
struct DrawLine {
    points: Vec<Vec2>,
    lifetime: f32,
}

#[derive(Component)]
struct SliderLine {
    timer: Timer,
}

fn draw_from_points(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut drawline_msg: ResMut<Messages<DrawLine>>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    for line in drawline_msg.drain() {
        // println!("Drawing-- {:?}", line.points);

        let mut asset = GizmoAsset::default();

        asset.linestrip_2d(line.points, Color::srgb(0.0, 1.0, 0.0));

        let handle = gizmo_assets.add(asset);

        commands.spawn((
            Gizmo {
                handle,
                line_config: GizmoLineConfig {
                    width: 3.0,
                    ..default()
                },
                ..Default::default()
            },
            SliderLine {
                timer: Timer::from_seconds(line.lifetime, TimerMode::Once),
            },
            Transform::default(),
        ));
    }
}

fn remove_line(time: Res<Time>, q: Query<(&mut SliderLine, Entity)>, mut commands: Commands) {
    for (mut slider_line, ent) in q {
        slider_line.timer.tick(time.delta());
        if slider_line.timer.is_finished() {
            commands.entity(ent).despawn();
        }
    }
}

#[derive(Message)]
struct LoadBeatmap {
    path: String,
}
fn load_osu_beatmap(
    mut bmap_msg: ResMut<Messages<LoadBeatmap>>,
    window: Single<&Window>,
    mut osu: ResMut<OsuBeatmap>,
    mut bmw: ResMut<BeatmapWorkerInfo>,
    time: Res<Time>,
) {
    let mut bmap_path_opt: Option<String> = None;

    for lbmp in bmap_msg.drain() {
        bmap_path_opt = Some(lbmp.path);
    }

    let Some(bmap_path) = bmap_path_opt else {
        return;
    };
    info!("Beatmap LOADING...");

    let screen_size = window.size();

    let mut beatmap = osuparser::parse_osu_file(Path::new(&bmap_path)).unwrap();

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
    bmw.started_at = time.elapsed_secs() + 1.0;
    bmw.start = true;
}

#[derive(Resource, Default)]
struct BeatmapWorkerInfo {
    start: bool,
    started_at: f32,
    next: usize,
}

fn beatmap_worker(
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
    let next = &osu.hit_objects[bmw.next];
    if time_since_start < (next.time - osu.real_approach_time) {
        return;
    }
    bmw.next += 1;
    // println!("Doing smth");

    oho_msg.write(next.clone());
}

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
    app.add_message::<OsuHitObject>();
    app.add_message::<RemoveCircle>();
    app.add_message::<LoadBeatmap>();
    app.add_message::<DrawLine>();
    app.add_message::<StartMovingSlider>();

    app.init_gizmo_group::<LineGizmos>();

    app.insert_resource(MouseInfo::default());
    app.insert_resource(OsuBeatmap::default());
    app.insert_resource(BeatmapWorkerInfo::default());
    app.insert_resource(MovingSlidersRes::default());

    app.add_systems(Startup, setup_world);
    app.add_systems(
        Update,
        (
            shrink_ring,
            mouse_position_system,
            summon_circle,
            circle_click,
            remove_circle,
            load_osu_beatmap,
            beatmap_worker,
            draw_from_points,
            remove_line,
            move_slider,
        ),
    );

    app.run();
}
