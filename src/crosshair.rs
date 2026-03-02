use bevy::prelude::*;

pub fn spawn_crosshair(mut commands: Commands) {
    let crosshair_size: f32 = 5.0;

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(crosshair_size),
                height: Val::Px(crosshair_size),
                ..default()
            },
            BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
        ));
    });
}