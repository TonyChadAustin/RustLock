use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};

mod interaction;
mod stats;
mod engine;

use crate::stats::GameState;
use crate::stats::GameTimer;
use crate::stats::States;

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
            mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
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
    game_state: Res<GameState>,
    mut game_timer: ResMut<GameTimer>,
    time: Res<Time>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    game_timer.timer -= time.delta_secs();
}