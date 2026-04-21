use bevy::prelude::*;
use crate::public_resources::FpsGUI;


pub struct GameDebugPlugin;

impl Plugin for GameDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        app.add_systems(Startup, fps_component_add);
        app.add_systems(Update, show_fps);
    }
}

fn fps_component_add(mut commands: Commands) {
    let text = "";
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(4.0),
            right: Val::Percent(0.0),
            justify_content: JustifyContent::End,
            overflow: Overflow::visible(),
            max_width: Val::Px(0.0),
            ..default()
        },))
        .with_children(|builder| {
            builder.spawn((
                // Transform::from_xyz(0.0, 0.0, 5.0),
                FpsGUI,
                Text::new(text.to_string()),
                // TextFont::from_font_size(67.0),
                TextFont {
                    font_size: 60.0,
                    // font: asset_server.load("fonts/OptimusPrinceps.ttf"),
                    // font: load_embedded_asset!(&*asset_server, "eassets/fonts/OptimusPrinceps.ttf"),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                TextLayout::new_with_justify(Justify::Center).with_no_wrap(),
                TextShadow {
                    color: Color::srgb(0.0, 0.0, 0.0),
                    ..default()
                },
            ));
        });
}


use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
fn show_fps(mut fpsgui: Single<&mut Text, With<FpsGUI>>, diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
        && let Some(value) = fps.smoothed()
    {
        fpsgui.0 = format!("FPS: {:.0}", value);
    }
}

