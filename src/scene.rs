use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

use crate::enemy::*;
use crate::card::{
    Card, LastCardSpawned, Superpower, spawn_card
};
use crate::ui::GameTimer;

const ENEMIES_SPAWN_TIME:f32 = 8.0;
const CARDS_SPAWN_TIME:f32 = 20.0;

const TEXT_RED_COLOR:Color = Color::srgb_u8(253, 5, 5);
const TEXT_GOLDEN_COLOR:Color = Color::srgb_u8(255, 215, 0);

struct Score {
    sentence: &'static str,
    time_seconds: f32,
    sprite: &'static str,
}

// Show score at GameOver
const GAME_OVER_SCORE_BRONZE:Score = Score {
    sentence: "Did you just come out of your mom ? Git gud !",
    time_seconds: 5.0 * 60.0,
    sprite: "sprites/bronze-trophy.png",
};

const GAME_OVER_SCORE_SILVER: Score = Score {
    sentence: "Not bad, but your sister did better yesterday !",
    time_seconds: 10.0 * 60.0,
    sprite: "sprites/silver-trophy.png",
};

const GAME_OVER_SCORE_GOLD: Score = Score {
    sentence: "Bro, get out of your room, it's starting to smell !",
    time_seconds: 15.0 * 60.0,
    sprite: "sprites/gold-trophy.png",
};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(EnemySpawnTimer(Timer::from_seconds(ENEMIES_SPAWN_TIME, TimerMode::Repeating)))
            .insert_resource(CardSpawnTimer(Timer::from_seconds(CARDS_SPAWN_TIME, TimerMode::Repeating)))
            .add_systems(Startup, setup_scene)
            .add_systems(OnEnter(GameState::Playing), play_start_sound)
            .add_systems(Update, adjust_spawn_rate.run_if(in_state(GameState::Playing)))
            .add_systems(Update, spawn_entities)
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct MenuScreen;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Component)]
struct GameOverScreen;

#[derive(Resource)]
pub struct EnemySpawnTimer(pub Timer);

#[derive(Resource)]
pub struct CardSpawnTimer(pub Timer);

#[derive(Component)]
pub struct Ground;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {

    /*
    * Ground
    */

    commands.spawn((
        SceneRoot(asset_server.load("models/ground.glb#Scene0")),
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::splat(1.0),
            ..default()
        },
        Ground
    ));
    
    /*commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 4.0))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
            Transform::from_xyz(0.0, 2.0, 6.0),
        ));*/

    // test

    /*let texture_image: Handle<Image> = asset_server.load_with_settings(
        "sprites/coeur.png",
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.sampler = bevy::image::ImageSampler::nearest();
        }
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
        Transform::from_translation(Vec3::new(0.0, 1.7, 0.0)),
        Card {
            power: Superpower::ELEXIR,
        }
    ));

    let texture_image: Handle<Image> = asset_server.load_with_settings(
        "sprites/jump.png",
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.sampler = bevy::image::ImageSampler::nearest();
        }
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
        Transform::from_translation(Vec3::new(8.0, 1.7, -4.0)),
        Card {
            power: Superpower::JUMP,
        }
    ));

    let texture_image: Handle<Image> = asset_server.load_with_settings(
        "sprites/speed.png",
        |settings: &mut bevy::image::ImageLoaderSettings| {
            settings.sampler = bevy::image::ImageSampler::nearest();
        }
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
        Transform::from_translation(Vec3::new(-8.0, 1.7, 4.0)),
        Card {
            power: Superpower::SPEEP,
        }
    ));*/

    /*
    * TEST Enemies
    */

    /*commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skull.glb"))),
        Transform::from_translation(Vec3::new(4.0, 2.0, 0.0)),
        SkullEnemy {
            speed: 1.0,
            elite: true,
            ..default()
        }
    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skeleton.glb"))),
        Transform::from_translation(Vec3::new(-4.0, 1.98, 0.0)).with_scale(Vec3::splat(1.8)),
        SkeletonEnemy::default(),
    )).with_children(|parent| {
    // Hitbox invisible qui suit le skeleton
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.3, 1.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 1.0, 0.0, 0.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, -0.1, 0.0),
        ));
    });*/
    

    // Black skybox
    commands.insert_resource(ClearColor(Color::BLACK));
}

fn spawn_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut enemy_timer: ResMut<EnemySpawnTimer>,
    mut card_timer: ResMut<CardSpawnTimer>,
    enemy_assets: Res<EnemyAssets>,
    game_timer: Res<GameTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_card: ResMut<LastCardSpawned>
) {

    let elapsed = game_timer.0.elapsed_secs();

    if enemy_timer.0.tick(time.delta()).just_finished() {

        let skulls:i8;
        let elites:i8;
        let skeletons:i8;

        if elapsed <= 30.0 {
            skulls = 2;
            elites = 0;
            skeletons = 0;
        } else if elapsed <= 150.0 {
            skulls = 2;
            elites = 0;
            skeletons = 1;
        } else if elapsed <= 300.0 {
            skulls = 3;
            elites = 0;
            skeletons = 1;
        } else if elapsed <= 600.0 {
            skulls = 4;
            elites = 2;
            skeletons = 1;
        } else {
            skulls = 5;
            elites = 2;
            skeletons = 3;
        }

        /*let skulls = 1 + (elapsed / 20.0) as u32; // more enemies
        let elites = 1 + (elapsed / 120.0) as u32;
        let skeletons = 1 + (elapsed / 120.0) as u32;*/

        // TRACE
        /*info!("{} skulls pop !",skulls);
        info!("{} skulls pop !",elites);
        info!("{} skulls pop !",skeletons);
        info!("===============================");*/
        
        let mut rng = rand::rng();

        for _ in 0..skulls {

            let x_rng = rng.random_range(-20.0..20.0);
            let y_rng = rng.random_range(4.0..20.0);
            let z_rng = rng.random_range(-20.0..20.0);
            let speed_rng = rng.random_range(2.0..5.00);

            commands.spawn((
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skull.glb"))),
                Transform::from_translation(Vec3::new(x_rng, y_rng, z_rng)),
                SkullEnemy {
                    speed: speed_rng,
                    ..default()
                }
            ));

        }

        /*
         * Elite (red skull)
         */

        /*let elite_chance:f32 = if elapsed <= 300.0 { // 5 min barrier
            0.2
        } else if elapsed > 300.0 && elapsed <= 600.0 { // 10 min barrier
            0.5
        } else { // nightmare, 100% elite
            1.0 
        };*/

        /*let roll = rng.random_range(0.0..1.0);
        let is_elite = roll < elite_chance;

        info!("Tirage = {}",roll);
        if is_elite {
            info!("{} elites pop !",elites);
        }*/

        //if is_elite {

        for _ in 0..elites {

            let x_rng = rng.random_range(-20.0..20.0);
            let y_rng = rng.random_range(4.0..20.0);
            let z_rng = rng.random_range(-20.0..20.0);
            let speed_rng = rng.random_range(2.0..5.00);

            commands.spawn((
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skull.glb"))),
                Transform::from_translation(Vec3::new(x_rng, y_rng, z_rng)),
                SkullEnemy {
                    speed: speed_rng,
                    elite: true,
                    life: SkullEnemy::default().life + SkullEnemy::default().extra_life,
                    ..default()
                }
            ));

        }

        //}

        for _ in 0..skeletons {

            let x_rng = rng.random_range(-20.0..20.0);
            let z_rng = rng.random_range(-20.0..20.0);

            commands.spawn((
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skeleton.glb"))),
                Transform::from_translation(Vec3::new(x_rng, 1.98, z_rng)).with_scale(Vec3::splat(1.8)),
                SkeletonEnemy::default(),
                )).with_children(|parent| {
                // Hitbox invisible
                    parent.spawn((
                        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.2))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.0, 1.0, 0.0, 0.0),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_xyz(0.0, -0.1, 0.0),
                    ));
                });

        }

        // Play sound
        let sounds = [
            &enemy_assets.spawn_enemy_1,
            &enemy_assets.spawn_enemy_2,
            &enemy_assets.spawn_enemy_3,
            &enemy_assets.spawn_enemy_4,
            &enemy_assets.spawn_enemy_5,
        ];

        let sound = sounds[rng.random_range(0..sounds.len())];

        commands.spawn((
            AudioPlayer::new(sound.clone()),
            PlaybackSettings::DESPAWN,
        ));

    }

    if card_timer.0.tick(time.delta()).just_finished() {

        let lucky:f32;

        if elapsed <= 150.0 {
            lucky = 10.0;
        } else if elapsed <= 300.0 {
            lucky = 25.0;
        } else if elapsed <= 600.0 {
            lucky = 50.0;
        } else {
            lucky = 100.0;
        }

        let mut rng = rand::rng();
        let roll = rng.random_range(0.0..1.0);
        
        if roll < lucky { // You're lucky !
            
            let random_card = loop {

                let candidate = match rng.random_range(0..3) {
                    0 => Superpower::ELEXIR,
                    1 => Superpower::SPEEP,
                    _ => Superpower::JUMP,
                };

                // Prevent twice in a row with the same card
                if Some(candidate) != last_card.last_power {
                    break candidate;
                }

            };
            last_card.last_power = Some(random_card);

            let x_rng = rng.random_range(-20.0..20.0);
            let z_rng = rng.random_range(-20.0..20.0);

            spawn_card(
                &mut commands, 
                &mut meshes, 
                &mut materials, 
                &asset_server, 
                Vec3::new(x_rng, 1.7, z_rng), 
                random_card,
            );

        }
    }
}

fn adjust_spawn_rate(
    mut enemy_timer: ResMut<EnemySpawnTimer>,
    mut card_timer: ResMut<CardSpawnTimer>,
    game_timer: Res<GameTimer>,
) {
    let elapsed = game_timer.0.elapsed_secs();
    
    let enemy_duration = (ENEMIES_SPAWN_TIME - (elapsed / 120.0)).max(2.0);
    enemy_timer.0.set_duration(Duration::from_secs_f32(enemy_duration));

    let card_duration = (CARDS_SPAWN_TIME - (elapsed / 100.0)).max(8.0);
    card_timer.0.set_duration(Duration::from_secs_f32(card_duration));
}

fn setup_game_over(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    game_timer: Res<GameTimer>,
){

    let elapsed = game_timer.0.elapsed_secs();
    let minutes = (elapsed / 60.0) as u32;
    let seconds = (elapsed % 60.0) as u32;

    let rank = if elapsed >= GAME_OVER_SCORE_GOLD.time_seconds {
        Some(GAME_OVER_SCORE_GOLD)
    } else if elapsed >= GAME_OVER_SCORE_SILVER.time_seconds {
        Some(GAME_OVER_SCORE_SILVER)
    } else if elapsed >= GAME_OVER_SCORE_BRONZE.time_seconds {
        Some(GAME_OVER_SCORE_BRONZE)
    } else {
        None
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GameOverScreen,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 100.0,
                ..default()
            },
            TextColor(TEXT_RED_COLOR),
        ));

        // Show time
        parent.spawn((
            Text::new(format!("Time : {:02}:{:02}", minutes, seconds)),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
        ));
        
        // Show score
        if let Some(score) = rank {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Text::new(format!("Score : ")),
                    TextFont {
                        font: asset_server.load("fonts/alagard.ttf"),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(TEXT_GOLDEN_COLOR),
                ));
                row.spawn((
                    ImageNode::new(asset_server.load(score.sprite)),
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        margin: UiRect::right(Val::Px(10.0)),
                        ..default()
                    },
                ));
                row.spawn((
                    Text::new(score.sentence),
                    TextFont {
                        font: asset_server.load("fonts/alagard.ttf"),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(TEXT_GOLDEN_COLOR),
                ));
            });
        }

        // Retry
        parent.spawn((
            Text::new("Press Enter to retry"),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 30.0,
                ..default()
            },
            TextColor(TEXT_RED_COLOR),
            Node {
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
        ));
    });
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        MenuScreen,
    )).with_children(|parent| {
        
        parent.spawn((
            Text::new("Rust in peace"),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 80.0,
                ..default()
            },
            TextColor(TEXT_RED_COLOR),
        ));

        parent.spawn((
            Text::new("Score :"),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect {
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn((
                ImageNode::new(asset_server.load(GAME_OVER_SCORE_BRONZE.sprite)),
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    margin: UiRect::right(Val::Px(10.0)),
                    ..default()
                },
            ));
            row.spawn((
                Text::new(format!(" : 5 min ")),
                TextFont {
                    font: asset_server.load("fonts/alagard.ttf"),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect {
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn((
                ImageNode::new(asset_server.load(GAME_OVER_SCORE_SILVER.sprite)),
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    margin: UiRect::right(Val::Px(10.0)),
                    ..default()
                },
            ));
            row.spawn((
                Text::new(format!(" : 10 min")),
                TextFont {
                    font: asset_server.load("fonts/alagard.ttf"),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect {
                top: Val::Px(10.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            ..default()
        }).with_children(|row| {
            row.spawn((
                ImageNode::new(asset_server.load(GAME_OVER_SCORE_GOLD.sprite)),
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    margin: UiRect::right(Val::Px(10.0)),
                    ..default()
                },
            ));
            row.spawn((
                Text::new(format!(" : 15 min")),
                TextFont {
                    font: asset_server.load("fonts/alagard.ttf"),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn((
            Text::new("Press Enter to play"),
            TextFont {
                font: asset_server.load("fonts/alagard.ttf"),
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::srgb_u8(180, 180, 180)),
            Node {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            },
        ));
    });
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn play_start_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/entry.ogg")),
        PlaybackSettings::DESPAWN,
    ));
}





