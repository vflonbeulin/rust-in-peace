use bevy::prelude::*;

use crate::player::PlayerArms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Fps,
    Free,
}

#[derive(Component)]
pub struct FpsCamera {
    pub sensitivity: f32,
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub mode: CameraMode,
    pub velocity_y: f32,
    pub is_grounded: bool,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            speed: 10.0,
            yaw: 0.0,
            pitch: 0.0,
            mode: CameraMode::Fps,
            velocity_y: 0.0,
            is_grounded: true,
        }
    }
}

pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_arms_visibility);
    }
}


fn toggle_arms_visibility(
    camera_query: Query<&FpsCamera>,
    mut arms_query: Query<&mut Visibility, With<PlayerArms>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    for mut visibility in &mut arms_query {
        *visibility = if camera.mode == CameraMode::Fps {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

