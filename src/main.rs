mod player;
mod camera;
mod scene;
mod debug;
mod input;
mod crosshair;
mod ui;
mod tracer;
mod enemy;
mod pixelate;
mod card;
mod animation;
mod effect;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy::audio::Volume;
use player::PlayerPlugin;
use camera::FpsCameraPlugin;
use scene::ScenePlugin;
use debug::DebugPlugin;
use input::ControlsPlugin;
use ui::UiPlugin;
use enemy::EnemyPlugin;
use pixelate::PixelatePlugin;
use bevy::window::WindowMode;
use card::CardPlugin;
use bevy::window::MonitorSelection;
use bevy::window::VideoModeSelection;
use animation::AnimationPlugin;
use effect::EffectPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_cursor_options: Some(CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            }),
            primary_window: Some(Window {
                resolution: WindowResolution::new(1980, 1024),
                //resolution: WindowResolution::new(1024, 768),
                //mode: WindowMode::Fullscreen(MonitorSelection::Primary,VideoModeSelection::Current),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PlayerPlugin)
        .add_plugins(FpsCameraPlugin)
        .add_plugins(ControlsPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(CardPlugin)
        .add_plugins(AnimationPlugin)
        .add_plugins(PixelatePlugin)
        .add_plugins(EffectPlugin)
        .add_systems(Startup, setup_music)
        .run();
}

fn setup_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/whispers.ogg")),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2)),
    ));
}
