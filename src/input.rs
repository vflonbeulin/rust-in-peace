use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseButton;
use bevy::window::{
    PrimaryWindow,
    CursorGrabMode,
    CursorOptions
};
use bevy::ecs::system::SystemParam;
use rand::Rng;
use std::time::Duration;

use crate::camera::{FpsCamera, CameraMode};
use crate::scene::{GameState, Ground};
use crate::player::{Player, ArmsAnimations,ArmsAnimationPlayer};
use crate::tracer::{BulletTracer,TracerAssets,TracerPlugin};
use crate::enemy::{SkullEnemy, SkeletonEnemy,EnemyAssets};
use crate::effect::{HitFlash, BloodAssets, BloodParticle, spawn_blood_explosion};


use bevy::picking::mesh_picking::ray_cast::RayCastVisibility;
use bevy::picking::mesh_picking::ray_cast::MeshRayCastSettings as RayCastSettings;

const RAISE_HALFWAY_TIME: f32 = 0.60;

// Jump
const GRAVITY: f32 = 20.0;
const JUMP_FORCE: f32 = 6.0;
const GROUND_Y: f32 = 2.0;

// Game over
const FALL_DEATH_Y: f32 = -100.0;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub enum AttackState {
    #[default]
    Idle,           
    Raising,
    Attacking,
    Lowering,
}

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttackState>()
            .init_resource::<FireRate>()
            .add_systems(Update, camera_look)
            .add_systems(Update, camera_movement.run_if(in_state(GameState::Playing)))
            .add_systems(Update, toggle_grab_cursor)
            .add_systems(Update, toggle_camera_mode.run_if(in_state(GameState::Playing)))
            .add_systems(Update, attack_state_machine.run_if(in_state(GameState::Playing)))
            .add_plugins(TracerPlugin)
            .add_systems(Update, exit_game)
            .add_systems(Update, reset_input.run_if(in_state(GameState::GameOver).or(in_state(GameState::Menu))));

        #[cfg(target_os = "linux")]
        app.add_systems(Update, force_cursor_recenter);
    }
}

#[derive(Resource)]
pub struct FireRate {
    pub timer: Timer,
}

impl Default for FireRate {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating), // 10 tirs/seconde
        }
    }
}

#[derive(SystemParam)]
pub struct EnemyQueries<'w, 's> {
    pub skulls: Query<'w, 's, (Entity, &'static mut SkullEnemy), Without<SkeletonEnemy>>,
    pub skeletons: Query<'w, 's, (Entity, &'static mut SkeletonEnemy), Without<SkullEnemy>>,
    pub parents: Query<'w, 's, &'static ChildOf>,
    pub transforms: Query<'w, 's, &'static Transform>,
    pub enemy_asset: Res<'w, EnemyAssets>,
}

fn camera_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FpsCamera)>,
) {
    let Ok((mut transform, mut camera)) = query.single_mut() else {
        return;
    };

    for motion in mouse_motion.read() {
        let delta = motion.delta;

        camera.yaw -= delta.x * camera.sensitivity;
        camera.pitch -= delta.y * camera.sensitivity;

        camera.pitch = camera.pitch.clamp(-1.54, 1.54);

        transform.rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw)
            * Quat::from_axis_angle(Vec3::X, camera.pitch);
    }
}

fn camera_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut FpsCamera)>,
    time: Res<Time>,
    mut ray_cast: MeshRayCast,
    mut player_query: Query<&mut Player>,
    ground_query: Query<Entity, With<Ground>>,
    parents: Query<&ChildOf>,
) {
    
    let Ok((mut transform, mut camera)) = query.single_mut() else { return;};
    let Ok(mut player) = player_query.single_mut() else { return; };

    let mut direction = Vec3::ZERO;

    let (forward, right) = match camera.mode {
        CameraMode::Fps => {
            
            let mut fwd = transform.rotation * Vec3::NEG_Z;
            fwd.y = 0.0; 
            fwd = fwd.normalize_or_zero();
            let right = transform.rotation * Vec3::X;
            (fwd, right)
        },
        CameraMode::Free => {
            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            (forward, right)
        },
    };

    // fr or international keyboard
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::KeyZ) {
        direction += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::KeyQ) {
        direction -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right;
    }

    // If fps mode, do not allow the player to move up and down
    if camera.mode == CameraMode::Free {
        if keyboard.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }
    } else { // Fps mode

        let ray = Ray3d::new(transform.translation, Dir3::NEG_Y);
        
        // Prevent frustum culling
        let hits = ray_cast.cast_ray(ray, &RayCastSettings::default()
            .with_visibility(RayCastVisibility::Any)
        );

        // check only ground
        let over_ground = hits.iter()
            .filter(|(entity, _)| {
                let mut current = *entity;
                loop {
                    if ground_query.contains(current) {
                        return true;
                    }
                    if let Ok(child_of) = parents.get(current) {
                        current = child_of.parent();
                    } else {
                        return false;
                    }
                }
            })
            .next()
            .map(|(_, hit)| hit.distance < 3.0)
            .unwrap_or(false);

        //info!("over_groud = {}",over_ground);

        let jump_force: f32;
        let gravity: f32;

        // Superpower jump card
        if player.has_super_jump {
            jump_force = JUMP_FORCE + 14.0;
            gravity = GRAVITY - 4.0;
        } else {
            // default
            jump_force = JUMP_FORCE;
            gravity = GRAVITY;
        }

        if keyboard.just_pressed(KeyCode::Space) && camera.is_grounded {
            camera.velocity_y = jump_force; 
            camera.is_grounded = false;
        }

        camera.velocity_y -= gravity * time.delta_secs();
        transform.translation.y += camera.velocity_y * time.delta_secs();

        if transform.translation.y <= GROUND_Y && camera.velocity_y <= 0.0 && over_ground {
            transform.translation.y = GROUND_Y;
            camera.velocity_y = 0.0;
            camera.is_grounded = true;
        }
        //transform.translation.y = GROUND_Y;

        // Player fall ? => GameOver bro
        if transform.translation.y <= FALL_DEATH_Y {player.current_life = 0.0;}
    
    }

    // Superpower speed card
    let speed = if player.has_speed {
        camera.speed * 2.0
    } else {
        camera.speed
    };

    if direction.length() > 0.0 {
        direction = direction.normalize();
       transform.translation += direction * speed * time.delta_secs();
    }
}

#[cfg(target_os = "linux")]
fn force_cursor_recenter(
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.single_mut() else { return };
    let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    window.set_cursor_position(Some(center));
}

// To be modified later ...
fn toggle_grab_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match cursor_options.grab_mode {
            CursorGrabMode::Locked => {
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
            _ => {
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
        }
    }
}

fn toggle_camera_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut FpsCamera, &mut Transform)>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        if let Ok((mut camera, mut transform)) = query.single_mut() {
            camera.mode = match camera.mode {
                CameraMode::Fps => {
                    cursor_options.visible = true;
                    CameraMode::Free
                },
                CameraMode::Free => {
                    transform.translation.y = 2.0;
                    cursor_options.visible = false;
                    CameraMode::Fps
                }
            };

            info!("Mode caméra: {:?}", camera.mode);
        }
    }
}

fn attack_state_machine(
    mouse_input: Res<ButtonInput<MouseButton>>,
    arms_animations: Res<ArmsAnimations>,
    mut players: Query<&mut AnimationPlayer, With<ArmsAnimationPlayer>>,
    mut state: ResMut<AttackState>,
    mut ray_cast: MeshRayCast,
    camera_query: Query<(&Camera, &GlobalTransform, &Player), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut enemies: EnemyQueries,
    mut commands: Commands,
    tracer_assets: Res<TracerAssets>,
    mut fire_rate: ResMut<FireRate>,
    time: Res<Time>,
    blood_assets: Res<BloodAssets>,
) {

    // Update timer
    fire_rate.timer.tick(time.delta());

    let Ok((camera, cam_transform, player)) = camera_query.single() else { return };

    if player.has_speed {
        fire_rate.timer.set_duration(Duration::from_secs_f32(0.06));
    } else {
        fire_rate.timer.set_duration(Duration::from_secs_f32(0.1)); // default
    }

    for mut anim_player in &mut players {
        
        match *state {
            AttackState::Idle => {
                
                if mouse_input.just_pressed(MouseButton::Left) {
                    anim_player.stop(arms_animations.idle_index);
                    anim_player.play(arms_animations.raise_attack_index);
                    *state = AttackState::Raising;
                }
            }
            AttackState::Raising => {
                
                // Accelerate animation for shooting
                if let Some(anim) = anim_player.animation_mut(arms_animations.raise_attack_index) {
                    anim.set_speed(2.0); 
                }

                let current_time = anim_player
                    .animation(arms_animations.raise_attack_index)
                    .map(|a| a.seek_time())
                    .unwrap_or(0.0);

                if current_time >= RAISE_HALFWAY_TIME {
                    if mouse_input.pressed(MouseButton::Left) {
                        
                        anim_player.stop(arms_animations.raise_attack_index);
                        anim_player.play(arms_animations.attack_index).repeat();
                        *state = AttackState::Attacking;

                    } else {
                        if let Some(anim) = anim_player.animation_mut(arms_animations.raise_attack_index) {
                            anim.set_speed(-1.0);
                        }
                        *state = AttackState::Lowering;
                    }
                } else if mouse_input.just_released(MouseButton::Left) {
                    if let Some(anim) = anim_player.animation_mut(arms_animations.raise_attack_index) {
                        anim.set_speed(-1.0);
                    }
                    *state = AttackState::Lowering;
                }
            }
            AttackState::Attacking => {
            
                if mouse_input.pressed(MouseButton::Left) && fire_rate.timer.just_finished() {

                    let Ok(window) = window_query.single() else { return };

                    // Raycast
                    let Ok(ray) = camera.viewport_to_world(cam_transform, Vec2::new(window.width() / 2.0, window.height() / 2.0)) else { return };
                    //let ray = Ray3d::new(cam_transform.translation, cam_transform.forward());
                    let hits = ray_cast.cast_ray(ray, &default());

                    const MAX_RANGE: f32 = 100.0;
                    
                    let hit_data_point = if let Some((_entity, hit_data)) = hits.first() {
                        hit_data.point
                    } else {
                        ray.origin + ray.direction * MAX_RANGE
                    };
                    
                    // We spawn the projectiles, side by side

                    commands.spawn((
                        Mesh3d(tracer_assets.mesh.clone()),
                        MeshMaterial3d(tracer_assets.material_yellow.clone()),
                        Transform::from_translation(Vec3::splat(f32::MAX)),
                        BulletTracer::new(
                            cam_transform.translation() + cam_transform.right() * 0.5 + cam_transform.forward() * 0.8,
                            hit_data_point, 
                            100.0
                        )
                    ));
                    
                    // Change color is player has superpower speed

                    let mesh_material_3d = if player.has_speed {
                        tracer_assets.material_red.clone()
                    } else {
                        tracer_assets.material_blue.clone()
                    };

                    commands.spawn((
                        Mesh3d(tracer_assets.mesh.clone()),
                        MeshMaterial3d(mesh_material_3d),
                        Transform::from_translation(Vec3::splat(f32::MAX)),
                        BulletTracer::new(
                            cam_transform.translation() + cam_transform.right() * 0.4 + cam_transform.forward() * 0.8, 
                            hit_data_point,
                            100.0
                        )
                    ));

                    if player.has_speed {

                        commands.spawn((
                            AudioPlayer::new(tracer_assets.speed_shoot_sound.clone()),
                            PlaybackSettings::DESPAWN,
                        ));

                    } else {

                        commands.spawn((
                            AudioPlayer::new(tracer_assets.shoot_sound.clone()),
                            PlaybackSettings::DESPAWN,
                        ));

                    }
                    

                    // Need to move up the hierarchy, touch parent, not child (mesh)
                    for (entity, _hit) in hits.iter() {
                        let mut current = *entity;
                        loop {
                            
                            // Check SkullEnemy
                            if let Ok((_,mut skull)) = enemies.skulls.get_mut(current) {

                                skull.life -=10;
                                commands.spawn((
                                    AudioPlayer::new(tracer_assets.hit_sound.clone()),
                                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.5)),
                                ));


                                if skull.life <= 0 {
                                    if let Ok(transform) = enemies.transforms.get(current) {

                                        spawn_blood_explosion(&mut commands, &blood_assets, transform.translation);

                                        commands.spawn((
                                            AudioPlayer::new(enemies.enemy_asset.skull_explode.clone()),
                                            PlaybackSettings::DESPAWN,
                                        ));

                                    }
                                    commands.entity(current).despawn();
                                } else {
                                    commands.entity(current).insert(HitFlash::default());
                                }
                                break;
                            } 

                            // Check SkeletonEnemy
                            if let Ok((_, mut skeleton)) = enemies.skeletons.get_mut(current) {

                                skeleton.life -= 10;
                                commands.spawn((
                                    AudioPlayer::new(tracer_assets.hit_sound.clone()),
                                    PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.5)),
                                ));
                                
                                if skeleton.life <= 0 {
                                    if let Ok(transform) = enemies.transforms.get(current) {

                                        spawn_blood_explosion(&mut commands, &blood_assets, transform.translation);

                                        commands.spawn((
                                            AudioPlayer::new(enemies.enemy_asset.skeleton_explode.clone()),
                                            PlaybackSettings::DESPAWN,
                                        ));

                                    }
                                    commands.entity(current).despawn();
                                } else {
                                    commands.entity(current).insert(HitFlash::default());
                                }
                                break;
                            }
                            
                            if let Ok(child_of) = enemies.parents.get(current) {
                                current = child_of.parent();
                            } else {
                                break;
                            }
                        }
                    }
                }

                if mouse_input.just_released(MouseButton::Left) {
                    anim_player.stop(arms_animations.attack_index);
                    anim_player.play(arms_animations.raise_attack_index);
                    if let Some(anim) = anim_player.animation_mut(arms_animations.raise_attack_index) {
                        anim.set_speed(-1.0);
                        anim.seek_to(RAISE_HALFWAY_TIME);
                    }
                    *state = AttackState::Lowering;
                }
            }
            AttackState::Lowering => {
                
                if mouse_input.pressed(MouseButton::Left) && fire_rate.timer.just_finished() {
                    // Player wants to attack again, restart sequence
                    *state = AttackState::Raising;
                } else {
                    let lowering_finished = anim_player
                    .animation(arms_animations.raise_attack_index)
                    .map(|a| a.seek_time() <= 0.01)
                    .unwrap_or(true);

                    if lowering_finished {
                        anim_player.stop(arms_animations.raise_attack_index);
                        anim_player.play(arms_animations.idle_index).repeat();
                        *state = AttackState::Idle;
                    }
                }

                
            }
        }
    }
}

fn exit_game(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_exit_writer: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        app_exit_writer.write(AppExit::Success);
    }
}

fn reset_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    player_query: Query<&Player>,
) {

    let Ok(player) = player_query.single() else { return; };

    // Player is death, restart level
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}


