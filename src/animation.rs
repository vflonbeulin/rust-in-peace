use bevy::prelude::*;

use crate::enemy::SkeletonEnemy;
use crate::player::{ArmsAnimations, ArmsAnimationPlayer,PlayerArms};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_animations)
            .add_systems(Update, setup_animations_once_loaded);
    }
}

#[derive(Resource)]
pub struct EnemyAnimations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph_handle: Handle<AnimationGraph>,
}

fn setup_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(1).from_asset("models/skeleton.glb")),
    ]);

    let graph_handle = graphs.add(graph);
    commands.insert_resource(EnemyAnimations {
        animations: node_indices,
        graph_handle,
    });
}

fn setup_animations_once_loaded(
    arms_animations: Res<ArmsAnimations>,
    enemy_animations: Res<EnemyAnimations>,
    skeleton_query: Query<Entity, With<SkeletonEnemy>>,
    arms_query: Query<(Entity, &AnimationGraphHandle), With<PlayerArms>>,
    mut anim_player_query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parent_query: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (player_entity, mut animation_player) in &mut anim_player_query {

        let mut current = player_entity;

        while let Ok(child_of) = parent_query.get(current) {
            
            current = child_of.parent();

            if let Ok((_, graph_handle)) = arms_query.get(current) {
                commands.entity(player_entity)
                    .insert(graph_handle.clone())
                    .insert(ArmsAnimationPlayer); 
                animation_player.play(arms_animations.idle_index).repeat();
            }

            if skeleton_query.get(current).is_ok() {
                commands.entity(player_entity).insert(AnimationGraphHandle(enemy_animations.graph_handle.clone()));
                animation_player.play(enemy_animations.animations[0]).repeat();
            }
        }
    }
}
