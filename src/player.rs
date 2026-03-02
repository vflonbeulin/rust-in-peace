use bevy::prelude::*;
use petgraph::graph::NodeIndex;
use bevy::camera::Exposure;
use std::time::Duration;

use crate::card::{Card, Superpower, CardAssets};
use crate::{camera::FpsCamera, pixelate::PixelateSettings};
use crate::enemy::{SkeletonEnemy,SkullEnemy};
use crate::scene::GameState;

const START_POSITION_PLAYER: Vec3 = vec3(0.0, 2.0, 5.0);
const ARMS_SCALE_FACTOR:f32 = 1.375;
const TIME_SECONDS_SUPERPOWER: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_player)
            //.add_systems(Update, enemy_contact_damage.run_if(in_state(GameState::Playing)))
            .add_systems(Update, check_player_death.run_if(in_state(GameState::Playing)))
            //.add_systems(Update,card_contact)
            .add_systems(Update, check_for_collisions.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Playing), reset_player)
            .add_systems(OnEnter(GameState::GameOver), on_game_over)
            .init_resource::<DamageCooldown>()
            .init_resource::<SuperpowerCooldown>()
            .add_message::<PlayerHitEvent>()
            .add_message::<CardPickedEvent>();
    }
}

#[derive(Resource)]
pub struct ArmsAnimations {
    pub idle_index: NodeIndex,
    pub attack_index: NodeIndex,
    pub raise_attack_index: NodeIndex
}

#[derive(Component)]
pub struct ArmsAnimationPlayer;

#[derive(Component)]
pub struct PlayerArms;

#[derive(Component)]
pub struct Player {
    pub current_life: f32,
    pub max_life: f32,
    pub has_super_jump: bool,
    pub has_speed: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            current_life: 100.0,
            max_life: 100.0,
            has_super_jump: false,
            has_speed: false
        }
    }
}

#[derive(Resource)]
pub struct DamageCooldown {
    pub timer: Timer,
}

impl Default for DamageCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

#[derive(Resource)]
pub struct SuperpowerCooldown {
    pub timer: Timer,
}

impl Default for SuperpowerCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(TIME_SECONDS_SUPERPOWER, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(TIME_SECONDS_SUPERPOWER));
        Self { timer }
    }
}



#[derive(Message)]
pub struct PlayerHitEvent;

#[derive(Message)]
pub struct CardPickedEvent {
    pub power: Superpower,
}


fn init_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    /*mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,*/
) {

    /* 
     * Arms animations 
     */
   
    let mut graph = AnimationGraph::default();

    let idle_index = graph.add_clip(
        asset_server.load("models/arms.glb#Animation9"),
        1.0,
        graph.root,
    );

    let raise_attack_index: NodeIndex = graph.add_clip(
        asset_server.load("models/arms.glb#Animation6"),
        1.0,
        graph.root,
    );

    let attack_index: NodeIndex = graph.add_clip(
        asset_server.load("models/arms.glb#Animation7"),
        1.0,
        graph.root,
    );

    commands.insert_resource(ArmsAnimations {
        idle_index,
        attack_index,
        raise_attack_index
    });

    let graph_handle = graphs.add(graph);

    /* 
     * Camera to Player (FPS)
     */

    commands.spawn((
        Camera3d::default(),
        Exposure { ev100: 13.0 }, // Plus élevé = plus sombre
        IsDefaultUiCamera, // We need to specify this for render Ui.
        PixelateSettings{ block_size: 4.0},
        Transform::from_translation(START_POSITION_PLAYER),
        FpsCamera::default(),
        Player::default(),
    )).with_children(|parent| {
        // Arms attached to camera
        parent.spawn(( 
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/arms.glb"))),
            Transform::from_xyz(0.0, -2.353, -0.196) // z=-0.196 // y = -2.353
                .with_scale(Vec3::splat(ARMS_SCALE_FACTOR))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            AnimationGraphHandle(graph_handle),
            PlayerArms,
        ));
        parent.spawn((SpotLight {
            intensity: 3_000_000.0, 
            range: 50.0, 
            outer_angle: 3.0,
            inner_angle: 2.8,
            ..Default::default()
        },
            Transform::from_xyz(0.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });

}

fn check_player_death(
    player_query: Query<&Player>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(player) = player_query.single() else { return; };
    if player.current_life <= 0.0  {
        next_state.set(GameState::GameOver);
    }
}

fn reset_player(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    mut arms_query: Query<&mut Transform, (With<PlayerArms>, Without<Player>)>,
) {
    let Ok((mut player, mut transform)) = player_query.single_mut() else { return; };
    player.current_life = player.max_life;
    transform.translation = START_POSITION_PLAYER;

    if let Ok(mut arms_transform) = arms_query.single_mut() {
        arms_transform.scale = Vec3::splat(ARMS_SCALE_FACTOR);
    }
}

fn on_game_over(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut arms_query: Query<&mut Transform, (With<PlayerArms>, Without<Player>)>,
) {
    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation.y -= 1.5;
    }
    if let Ok(mut arms_transform) = arms_query.single_mut() {
        arms_transform.scale = Vec3::ZERO;
    }

    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/gameover.ogg")),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.7)),
    ));
}

fn check_for_collisions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(&Transform, &mut Player)>,
    card_query: Query<(Entity, &Transform, &Card), With<Card>>,
    enemies: Query<&Transform, Or<(With<SkullEnemy>, With<SkeletonEnemy>)>>,
    mut damage_cooldown: ResMut<DamageCooldown>,
    mut superpower_cooldown: ResMut<SuperpowerCooldown>,
    time: Res<Time>,
    mut hit_events: MessageWriter<PlayerHitEvent>,
    mut card_events: MessageWriter<CardPickedEvent>,
    card_assets: Res<CardAssets>,
) {

    // Without cooldown, player loose x fps * 10 damages...
    damage_cooldown.timer.tick(time.delta());
    superpower_cooldown.timer.tick(time.delta());

    let Ok((player_transform, mut player)) = player_query.single_mut() else { return; };
    
    if superpower_cooldown.timer.just_finished() {
        player.has_super_jump = false;
        player.has_speed = false;
    }

    // Check nearby ennemies 
    for enemy_transform in &enemies {
        let distance = player_transform.translation.distance(enemy_transform.translation);
        if distance < 1.5 && damage_cooldown.timer.is_finished() {
 
            player.current_life -= 30.0;
            if player.current_life <= 0.0 {player.current_life = 0.0;}
            
            damage_cooldown.timer.reset();
            hit_events.write(PlayerHitEvent); 

            commands.spawn((
                AudioPlayer::new(asset_server.load("sounds/damage.ogg")),
                PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(0.5)),
            ));
        }
    }

    // Check nearby cards
    for(card_entity,card_transform, card) in &card_query {

        let distance = player_transform.translation.distance(card_transform.translation);

        if distance < 1.5 {
            
            commands.entity(card_entity).despawn();

            match card.power {

                // Assumed that the cards cannot be simultaneous

                Superpower::ELEXIR => {
                    let new_life = player.current_life + 20.0;
                    if new_life > player.max_life {
                        player.current_life = player.max_life // max life
                    } else {
                        player.current_life = new_life;
                    }

                    commands.spawn((
                        AudioPlayer::new(card_assets.life_sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                },
                Superpower::JUMP => {
                    player.has_super_jump = true;
                    superpower_cooldown.timer.reset(); // Launch the timer for x seconds

                    commands.spawn((
                        AudioPlayer::new(card_assets.jump_sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                },
                Superpower::SPEEP => {
                    player.has_speed = true;
                    superpower_cooldown.timer.reset(); // Launch the timer for x seconds

                    commands.spawn((
                        AudioPlayer::new(card_assets.speed_sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }

            card_events.write(CardPickedEvent { power: card.power }); // Update Ui

        }

    }

    
}


