use bevy::prelude::*;

const ROTATION_SPEED:f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Superpower {
    ELEXIR,
    SPEEP,
    JUMP,
}

impl Superpower {
    pub fn color(&self) -> Color {
        match self {
            Superpower::ELEXIR => Color::srgb_u8(255, 0, 0),
            Superpower::SPEEP => Color::srgb_u8(35, 144, 99),
            Superpower::JUMP => Color::srgb_u8(56, 89, 179),
        }
    }
}

#[derive(Component)]
pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_card_assets)
            .init_resource::<LastCardSpawned>()
            .add_systems(Update,rotate_card);
    }
}

#[derive(Component)]
pub struct Card {
    pub power:Superpower
}

#[derive(Resource, Default)]
pub struct LastCardSpawned {
    pub last_power: Option<Superpower>,
}


#[derive(Resource)]
pub struct CardAssets {
    pub life_sound: Handle<AudioSource>,
    pub speed_sound: Handle<AudioSource>,
    pub jump_sound: Handle<AudioSource>,
}

pub fn setup_card_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(CardAssets {
        life_sound: asset_server.load("sounds/life.ogg"),
        speed_sound: asset_server.load("sounds/speed.ogg"), 
        jump_sound: asset_server.load("sounds/jump.ogg"),
    });
}

fn rotate_card(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Card>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * ROTATION_SPEED);
    }
}

pub fn spawn_card(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    position: Vec3,
    power: Superpower,
) {

    let texture_path:&str = match power {
        Superpower::ELEXIR => "sprites/coeur.png",
        Superpower::SPEEP => "sprites/speed.png",
        Superpower::JUMP => "sprites/jump.png",
    };      

    let texture_image: Handle<Image> = asset_server.load_with_settings(
        texture_path,
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.sampler = bevy::image::ImageSampler::nearest();
        },
    );
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(1.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_image),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_translation(position),
        Card { power },
    ));
}

