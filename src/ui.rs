use bevy::prelude::*;
use bevy::time::Stopwatch;

use super::crosshair;

use crate::player::{Player, PlayerHitEvent};
use crate::scene::GameState;
use crate::player::CardPickedEvent;
use crate::card::Superpower;

const WIDTH_BAR: f32 = 250.0;


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .insert_resource(GameTimer(Stopwatch::new()))
            .add_systems(Startup,crosshair::spawn_crosshair)
            .add_systems(Update, update_ui)
            .add_systems(Update, update_damage_flash)
            .add_systems(OnEnter(GameState::Playing), reset_damage_flash)
            .add_systems(Update, update_superpower_ui)
            .add_systems(Update, update_game_timer.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_superpower_screen)
            .add_systems(OnEnter(GameState::Playing), reset_game_timer);
    }
}

#[derive(Resource)]
pub struct GameTimer(pub Stopwatch);

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct TopHealthBar;

#[derive(Component)]
pub struct BotHealthBar;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct DamageFlash {
    pub timer: Timer,
}

#[derive(Component)]
pub struct SuperpowerFlash {
    pub timer: Timer,
}


#[derive(Component)]
pub struct SuperpowerText;


fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    
    let font = asset_server.load("fonts/alagard.ttf");

    commands.spawn((
        Text::new("00:00"),
        TextFont {
            font: font.clone(),
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Percent(50.0),
            ..default()
        },
        TimerText,
    ));

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(50.0),
        position_type: PositionType::Absolute,
        bottom: Val::Px(10.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|parent| {

        // Show Superpower
        parent.spawn((
            Text::new(""),
            TextFont {
                font: font.clone(),
                font_size: 50.0,
                ..default()
            },
            TextColor(Color::srgb_u8(56, 89, 179)),
            Node {
                margin: UiRect::right(Val::Px(15.0)),
                ..default()
            },
            SuperpowerText,
        ));

        parent.spawn((Node {
            width: Val::Px(WIDTH_BAR),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(80, 80, 80)),
        )).with_children(|bars| {
            // Top bar
            bars.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(15.0),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(253, 5, 5)),
                TopHealthBar,
            ));
            
            // Bot bar
            bars.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(187, 24, 47)),
                BotHealthBar,
            ));
        });
        
        // Texte HP
        parent.spawn((
            Text::new("100"),
            TextFont {
                font: font,
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::left(Val::Px(15.0)),
                ..default()
            },
            HealthText,
        ));
    });

    // Damage flash when enemy hit player
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)), // alpha 0 = invisible
        DamageFlash { timer: {
            let mut t = Timer::from_seconds(0.3, TimerMode::Once);
            t.tick(std::time::Duration::from_secs(1));
            t
        }},
    ));

    // Superpower screen effect
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        SuperpowerFlash {
            timer: {
                let mut t = Timer::from_seconds(0.6, TimerMode::Once);
                t.tick(std::time::Duration::from_secs(1));
                t
            },
        },
    ));

}

fn update_ui(
    player_query: Query<Ref<Player>>,
    mut bar_query: Query<(&mut Node, Option<&TopHealthBar>, Option<&BotHealthBar>)>,
    mut text_query: Query<&mut Text, With<HealthText>>,
) {
    
    let Ok(player) = player_query.single() else { return; };

    if player.is_changed() {
        
        let percent = (player.current_life as f32 / player.max_life as f32) * 100.0;

        for (mut node, top, bot) in &mut bar_query {
            if top.is_some() || bot.is_some() {
                node.width = Val::Percent(percent);
            }
        }
        
        if let Ok(mut text) = text_query.single_mut() {
            **text = format!("{} / {}", player.current_life, player.max_life);
        }
    }
    
}

fn update_damage_flash(
    mut flash_query: Query<(&mut BackgroundColor, &mut DamageFlash)>,
    mut hit_events: MessageReader<PlayerHitEvent>,
    time: Res<Time>,
) {
    let Ok((mut color, mut flash)) = flash_query.single_mut() else { return; };

    for _ in hit_events.read() {
        flash.timer.reset();
        *color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.3));
    }

    // Fade out progress
    flash.timer.tick(time.delta());
    if !flash.timer.is_finished() {
        let alpha = 0.3 * (1.0 - flash.timer.fraction());
        *color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, alpha));
    }
}

pub fn reset_damage_flash(
    mut flash_query: Query<(&mut BackgroundColor, &mut DamageFlash)>,
) {
    let Ok((mut color, mut flash)) = flash_query.single_mut() else { return; };
    *color = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0));
    flash.timer.tick(std::time::Duration::from_secs(1)); // force terminé
}

fn update_superpower_ui(
    mut card_events: MessageReader<CardPickedEvent>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SuperpowerText>>,
    player_query: Query<&Player>,
) {
    for event in card_events.read() {
        let Ok((mut text, mut color)) = text_query.single_mut() else { return; };
        match event.power {
            Superpower::JUMP => **text = "J".to_string(),
            Superpower::SPEEP => **text = "S".to_string(),
            _ => {},
        }
        *color = TextColor(event.power.color());
    }

    // Erase if superpower is finished
    let Ok(player) = player_query.single() else { return; };
    if !player.has_super_jump && !player.has_speed {
        if let Ok((mut text, _)) = text_query.single_mut() {
            if !text.is_empty() {
                **text = String::new();
            }
        }
    }
}

fn update_game_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut text_query: Query<&mut Text, With<TimerText>>,
) {
    game_timer.0.tick(time.delta());
    
    let elapsed = game_timer.0.elapsed_secs();
    let minutes = (elapsed / 60.0) as u32;
    let seconds = (elapsed % 60.0) as u32;

    if let Ok(mut text) = text_query.single_mut() {
        **text = format!("{:02}:{:02}", minutes, seconds);
    }
}

fn reset_game_timer(mut game_timer: ResMut<GameTimer>) {
    game_timer.0.reset();
}

fn update_superpower_screen(
    mut flash_query: Query<(&mut BackgroundColor, &mut SuperpowerFlash)>,
    mut card_events: MessageReader<CardPickedEvent>,
    time: Res<Time>,
) {
    
    let Ok((mut bg, mut flash)) = flash_query.single_mut() else { return };

    let intensity:f32 = 0.6;

    for event in card_events.read() {
        flash.timer.reset();
        let c = event.power.color();
        *bg = BackgroundColor(c.with_alpha(intensity));
    }

    flash.timer.tick(time.delta());
    if !flash.timer.is_finished() {
        let alpha = intensity * (1.0 - flash.timer.fraction());
        let current = bg.0.to_srgba();
        *bg = BackgroundColor(Color::srgba(current.red, current.green, current.blue, alpha));
    }
}



