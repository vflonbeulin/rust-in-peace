use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::camera::FpsCamera;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup_debug_ui)
            .add_systems(Update, update_debug_ui);
    }
}

#[derive(Component)]
struct DebugText;

fn setup_debug_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Debug Info"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        DebugText,
    ));
}

fn update_debug_ui(
    mut text_query: Query<&mut Text, With<DebugText>>,
    camera_query: Query<(&Transform, &FpsCamera)>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    if let Ok((transform, camera)) = camera_query.single() {
        let pos = transform.translation;
        text.0 = format!(
            "FPS: {:.0}\nMode: {:?}\nPosition: ({:.1}, {:.1}, {:.1})\nYaw: {:.2}° | Pitch: {:.2}°\nSpeed: {:.1}",
            fps,
            camera.mode,
            pos.x, pos.y, pos.z,
            camera.yaw.to_degrees(),
            camera.pitch.to_degrees(),
            camera.speed
        );
    }
}
