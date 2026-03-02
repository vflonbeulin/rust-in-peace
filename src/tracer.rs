use bevy::prelude::*;

pub struct TracerPlugin;

impl Plugin for TracerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_tracer_assets)
            .add_systems(Update,update_tracers);
    }
}

#[derive(Component)]
pub struct BulletTracer {
    pub start_position: Vec3,
    pub end_position: Vec3,
    pub lifetime:f32,
    pub time_alive:f32
}

impl BulletTracer {
    pub fn new(start:Vec3,end:Vec3,speed:f32) -> BulletTracer {
        BulletTracer {
            start_position: start,
            end_position: end,
            lifetime: Vec3::distance(start, end) / speed,
            time_alive: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct TracerAssets {
    pub mesh: Handle<Mesh>,
    pub material_yellow: Handle<StandardMaterial>,
    pub material_blue: Handle<StandardMaterial>,
    pub material_red: Handle<StandardMaterial>,
    pub shoot_sound: Handle<AudioSource>,
    pub speed_shoot_sound: Handle<AudioSource>,
    pub hit_sound: Handle<AudioSource>, 
}

pub fn setup_tracer_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(TracerAssets {
        mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
        material_yellow: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            unlit: true,
            ..default()
        }),
        material_blue: materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 1.0),
            unlit: true,
            ..default()
        }),
        material_red: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            unlit: true,
            ..default()
        }),
        shoot_sound: asset_server.load("sounds/shoot-test.ogg"),
        speed_shoot_sound: asset_server.load("sounds/speed-shoot.ogg"),
        hit_sound: asset_server.load("sounds/hit.ogg"), 
    });
}

/*#[derive(Component)]
pub struct TracerSpawnSpot;*/

fn update_tracers(
    mut commands: Commands,
    mut tracer_query: Query<(&mut BulletTracer,&mut Transform, Entity)>,
    time: Res<Time>
) {
    for(mut tracer, mut transform, entity) in tracer_query.iter_mut() {
        tracer.time_alive += time.delta_secs();

        transform.translation = Vec3::lerp(
            tracer.start_position,
            tracer.end_position,
            f32::clamp(tracer.time_alive / tracer.lifetime,0.0,1.0)
        );
        //transform.look_at(tracer.end_position,Vec3::Y);
        let direction = (tracer.end_position - tracer.start_position).normalize();
        transform.look_to(direction, Vec3::Y);

        if tracer.time_alive > tracer.lifetime {
            commands.entity(entity).despawn();
        }
    }
}