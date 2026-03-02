use bevy::prelude::*;
use rand::prelude::*;

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_blood_assets)
            .add_systems(Update, update_blood_particles);
    }
}

#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
    pub original_materials: Vec<(Entity, Handle<StandardMaterial>)>,
}

impl Default for HitFlash {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.10, TimerMode::Once),
            original_materials: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct BloodParticle {
    pub velocity: Vec3,
    pub lifetime: Timer,
}

#[derive(Resource)]
pub struct BloodAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn setup_blood_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(BloodAssets {
        mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.0, 0.0),
            unlit: true,
            ..default()
        }),
    });
}

fn update_blood_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BloodParticle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut particle, mut transform) in query.iter_mut() {
        particle.velocity.y -= 15.0 * time.delta_secs();
        transform.translation += particle.velocity * time.delta_secs();
        transform.scale = Vec3::splat(particle.lifetime.fraction_remaining());

        particle.lifetime.tick(time.delta());
        if particle.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_blood_explosion(
    commands: &mut Commands,
    blood_assets: &BloodAssets,
    position: Vec3,
) {
    let mut rng = rand::rng();
    for _ in 0..160 {
        let velocity = Vec3::new(
            rng.random_range(-10.0..10.0),
            rng.random_range(0.0..10.0),
            rng.random_range(-10.0..10.0),
        );
        commands.spawn((
            Mesh3d(blood_assets.mesh.clone()),
            MeshMaterial3d(blood_assets.material.clone()),
            Transform::from_translation(position),
            BloodParticle {
                velocity,
                lifetime: Timer::from_seconds(0.6, TimerMode::Once),
            },
        ));
    }
}

