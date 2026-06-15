use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};

mod interaction;
mod stats;
mod engine;

use crate::stats::{GameState, 
    GameTimer, 
    States,
    TimerText,
    WinScreen,
    LoseScreen,
    Player,
    Health,
    PLAYER_HP,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins
    .set(ImagePlugin::default_nearest())
    .set(WindowPlugin {
        primary_window: Some(Window {
            title: "Rustlock".to_string(),
            resolution: (2200.0, 1100.0).into(),
            resizable: false,
            present_mode: PresentMode::AutoVsync,
            //mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            mode: WindowMode::Windowed,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(interaction::InteractionPlugin);
    app.add_plugins(engine::EnginePlugin);
    app.add_plugins(stats::StatsPlugin);
    app.add_systems(Update, progress_game_timer);
    app.run();
}

fn progress_game_timer(
    mut game_state: ResMut<GameState>,
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
    mut text: Query<&mut Text2d, With<TimerText>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player: Query<(&mut Transform, &mut Health), With<Player>>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    game_timer.timer -= time.delta_secs();

    let rem_secs = game_timer.timer.max(0.0).ceil() as u32;

    if let Ok((mut transform, health)) = player.single_mut() { 
        if rem_secs <= 0 {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            game_state.game_state = States::Win; //YAHOO
            commands.spawn( (
                Sprite {
                    custom_size: Some(Vec2::new(2555.0, 1435.0)),
                    image: asset_server.load("win.png"),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1110.0),
                WinScreen,
            ));
        }

        if health.current <= 0 {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            game_state.game_state = States::Win; //YAHOO
            commands.spawn( (
                Sprite {
                    custom_size: Some(Vec2::new(2555.0, 1435.0)),
                    image: asset_server.load("lose.png"),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1110.0),
                LoseScreen,
            ));
        }
    }

    if let Ok(mut t) = text.single_mut() {
        **t = rem_secs.to_string();
    }
}