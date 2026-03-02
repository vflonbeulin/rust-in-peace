use bevy::prelude::*;

use crate::player::Player;
use crate::effect::{
    HitFlash,
};

#[derive(Component)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy_assets)
            .add_systems(Update, apply_enemy_texture)
            .add_systems(Update, move_enemies)
            .add_systems(Update, handle_hit_flash);
    }
}

#[derive(Component)]
pub struct SkullEnemy {
    pub speed: f32,
    pub elite: bool,
    pub life: i16,
    pub extra_life: i16,
}

impl Default for SkullEnemy {
    fn default() -> Self {
        Self {
            speed: 0.0,
            elite: false,
            life: 100,
            extra_life:100,
        }
    }
}

#[derive(Component)]
pub struct SkeletonEnemy {
    pub speed: f32,
    pub life: i16,
    pub redirect_timer: Timer,
    pub current_direction: Vec3,
}

impl Default for SkeletonEnemy {
    fn default() -> Self {
        Self {
            speed: 1.4,
            life: 300,
            redirect_timer: Timer::from_seconds(1.0, TimerMode::Repeating), 
            current_direction: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct TextureApplied;

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
}

#[derive(Resource)]
pub struct EnemyAssets {
    pub spawn_enemy_1: Handle<AudioSource>,
    pub spawn_enemy_2: Handle<AudioSource>,
    pub spawn_enemy_3: Handle<AudioSource>,
    pub spawn_enemy_4: Handle<AudioSource>,
    pub spawn_enemy_5: Handle<AudioSource>,
    pub skull_explode: Handle<AudioSource>,
    pub skeleton_explode: Handle<AudioSource>,
    pub skeleton_walk: Handle<AudioSource>,
}

fn setup_enemy_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load all assets for the scene
    commands.insert_resource(EnemyAssets {
        spawn_enemy_1: asset_server.load("sounds/spawn-enemy-1.ogg"),
        spawn_enemy_2: asset_server.load("sounds/spawn-enemy-2.ogg"), 
        spawn_enemy_3: asset_server.load("sounds/spawn-enemy-3.ogg"), 
        spawn_enemy_4: asset_server.load("sounds/spawn-enemy-4.ogg"), 
        spawn_enemy_5: asset_server.load("sounds/spawn-enemy-5.ogg"),
        skull_explode: asset_server.load("sounds/skull-explode.ogg"),
        skeleton_explode: asset_server.load("sounds/skeleton-explode.ogg"),
        skeleton_walk: asset_server.load("sounds/skeleton-walk.ogg"),
    });
}

fn apply_enemy_texture(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Children,&SkullEnemy), (With<SkullEnemy>, Without<TextureApplied>)>,
    children_query: Query<&Children>,
    mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    let texture: Handle<Image> = asset_server.load("textures/128xred-lich.png");

    for (entity, children,enemy) in enemy_query.iter() {

        if enemy.elite {
            if apply_texture_recursive(children, &children_query, &mesh_query, &mut materials, &texture, &mut commands) {
                commands.entity(entity).insert(TextureApplied);
            }
        }

        
    }
}

fn apply_texture_recursive(
    children: &Children,
    children_query: &Query<&Children>,
    mesh_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    texture: &Handle<Image>,
    commands: &mut Commands,
) -> bool {
    let mut applied = false;
    for child in children.iter() {
        if let Ok(mat_handle) = mesh_query.get(child) {
            if let Some(material) = materials.get(&mat_handle.0) {
                let mut new_material = material.clone();
                new_material.base_color_texture = Some(texture.clone());
                let new_handle = materials.add(new_material);
                commands.entity(child).insert(MeshMaterial3d(new_handle));
                applied = true;
            }
        }
        if let Ok(grandchildren) = children_query.get(child) {
            if apply_texture_recursive(grandchildren, children_query, mesh_query, materials, texture,commands) {
                applied = true;
            }
        }
    }
    applied
}

fn move_enemies(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<SkullEnemy>, Without<SkeletonEnemy>)>,
    mut skull_query: Query<(&mut Transform, &SkullEnemy), Without<SkeletonEnemy>>,
    mut skeleton_query: Query<(&mut Transform, &mut SkeletonEnemy), Without<SkullEnemy>>,
    mut commands: Commands,
    enemy_asset: Res<EnemyAssets>
) {
    let Ok(player_transform) = player_query.single() else { return };

    for (mut transform, enemy) in skull_query.iter_mut() {
        let direction = (player_transform.translation - transform.translation).normalize();
        transform.translation += direction * enemy.speed * time.delta_secs();
        transform.look_at(player_transform.translation, Vec3::Y);
    }

    for (mut transform, mut enemy) in skeleton_query.iter_mut() {

        enemy.redirect_timer.tick(time.delta());

        if enemy.redirect_timer.just_finished() || enemy.current_direction == Vec3::ZERO {

            let mut dir = player_transform.translation - transform.translation;
            dir.y = 0.0; // Unlike skulls, stay at Y-axis
            enemy.current_direction = dir.normalize();

            // Play walk sound
            commands.spawn((
                AudioPlayer::new(enemy_asset.skeleton_walk.clone()),
                PlaybackSettings::DESPAWN,
            ));

        }

        transform.translation += enemy.current_direction * enemy.speed * time.delta_secs();
        
        let look_target = transform.translation - enemy.current_direction;
        transform.look_at(look_target, Vec3::Y);
    }

}

fn handle_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut HitFlash, &Children)>,
    children_query: Query<&Children>,
    mesh_query: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut flash, children) in flash_query.iter_mut() {
        
        if flash.original_materials.is_empty() {
            collect_materials_recursive(
                children,
                &children_query,
                &mesh_query,
                &mut flash.original_materials,
            );
        }
        
        flash.timer.tick(time.delta());
        let intensity = flash.timer.fraction_remaining();
        
        for (mesh_entity, original_handle) in &flash.original_materials {
            if let Some(material) = materials.get(original_handle) {
                let mut new_material = material.clone();
                new_material.emissive = Color::srgb(intensity * 2.0, intensity * 2.0, 0.0).into();
                let new_handle = materials.add(new_material);
                if let Ok(mut entity_commands) = commands.get_entity(*mesh_entity) {
                    entity_commands.try_insert(MeshMaterial3d(new_handle));
                }
            }
        }
        
        if flash.timer.just_finished() {
            for (mesh_entity, original_handle) in &flash.original_materials {
                if let Ok(mut entity_commands) = commands.get_entity(*mesh_entity) {
                    entity_commands.try_insert(MeshMaterial3d(original_handle.clone()));
                }
            }
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<HitFlash>();
            }
        }
    }
}

fn collect_materials_recursive(
    children: &Children,
    children_query: &Query<&Children>,
    mesh_query: &Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    collected: &mut Vec<(Entity, Handle<StandardMaterial>)>,
) {
    for child in children.iter() {
        if let Ok((mesh_entity, mat_handle)) = mesh_query.get(child) {
            collected.push((mesh_entity, mat_handle.0.clone()));
        }
        if let Ok(grandchildren) = children_query.get(child) {
            collect_materials_recursive(grandchildren, children_query, mesh_query, collected);
        }
    }
}
