use bevy::prelude::*;
use std::collections::BTreeMap;
use crate::engine::despawn_boxes;

//TODO ADD SETTINGS MENU FOR DYNAMIC TESTING
//PLAYER STATS
pub const SPAWNABLE_RANGE: f32 = 200.0;

//GAME STATS
pub const GRID_SIZE: i32 = 1300; // 1300 minimum that looks clean, more might be better idk
pub const GAME_TIME: f32 = 600.0;

//ENEMY STATS
pub const ENEMY_SPAWNS: f32 = 10.0;
pub const ENEMY_CURRENT_HP: i32 = 5;
pub const ENEMY_MAX_HP: i32 = 5;
pub const ENEMY_ATTACK_STUN: f32 = 0.8;
pub const ENEMY_SPEED: f32 = 200.0;


pub const PLAYER_BUILD_STUN: f32 = 0.3;
pub const PLAYER_SPEED: f32 = 250.0;

//UI STATS
pub const PLAY_BUTTON_POS: (f32, f32, f32, f32) = (500.0, 250.0, 300.0, -200.0); //size, size, pos, pos
pub const QUIT_BUTTON_POS: (f32, f32, f32, f32) = (500.0, 250.0, 300.0, -500.0);

#[derive(Resource)]
pub struct CursorPos(pub Vec2, pub Option<Vec2>);

#[derive(Component)]
pub struct Player {
    pub stun_timer: f32
}

#[derive(Component)]
pub struct Map; // Just the background rn

#[derive(Component)]
pub struct Box; // Spawnable prefab

#[derive(Component)]
pub struct Grid { // Normalization of world pixels to 50x50
    pub xy: Vec2, 
}

#[derive(Component)]
pub struct Line; // Visual aid between 2 things


#[derive(Component)]
pub struct Enemy {
    pub id: usize,
    pub memory: Vec<(i32, i32)>,
    pub memory_recency: bool,
    pub target: Option<(i32, i32)>,
    //pub state_changed: bool,
    pub rage: bool,
    pub stun_timer: f32,
}

#[derive(Component)]
pub struct MenuScreen;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct QuitButton;

#[derive(Resource)]
pub struct PathfindTrigger {
    pub triggered: bool,
}

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthBarParent();

#[derive(Component, Default)]
pub struct SpriteSize(pub Vec2);

#[derive(Bundle)]
pub struct HealthBarBundle {
    pub sprite: Sprite,
    pub transform: Transform,
    pub health_bar: HealthBar,
    pub health_bar_parent: HealthBarParent,
}

#[derive(Resource)]
pub struct GameTimer {
    pub timer: f32,
}

pub enum States {
    StartMenu,
    SettingsMenu,
    Level1,
    Win,
    Lose,
}

#[derive(Resource)]
pub struct GameState {
    pub game_state: States,
    pub update_pathfind: bool,
    pub grid_pairs: BTreeMap<(i32, i32), bool>,
    //pub boxes: Vec<((i32, i32), (i32, i32))>, //((x, y), (size x, size y)) OLD
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            //update_line,
            spawn_enemies,
        ));
        app.add_systems(Update, spawn_health_bars.after(despawn_boxes));
    
    }
}

fn _update_line(
    player: Query<&Transform, With<Player>>,
    cursor_pos: Res<CursorPos>,
    mut line_query: Query<(&mut Transform, &mut Sprite), (With<Line>, Without<Player>, Without<Enemy>)>,
    game_state: Res<GameState>,
    mut commands: Commands
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    if let Ok(player_transform) = player.single() {
        let player_pos = player_transform.translation.truncate();


        if let Some(cursor) = cursor_pos.1 {
            let mut does_line_exist = false;

            let end = player_pos;
            let delta = end - cursor;
            //let length = delta.length();
            let length;
            if 200.0 < delta.length() {
                length = 200.0;
            }
            else {
                length = delta.length();
            }
            let angle = delta.y.atan2(delta.x);
            let circle = end + Vec2::new(angle.cos(), angle.sin()) * -length;
            let midpt = (circle + end) / 2.0;

            for (mut transform, mut sprite) in &mut line_query {
                does_line_exist = true;

                transform.translation = midpt.extend(2.0);
                transform.rotation = Quat::from_rotation_z(angle);
    
                sprite.custom_size = Some(Vec2::new(length, 3.0));
            }
            if !does_line_exist {
                commands.spawn((
                    Sprite {
                        color: Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
                        custom_size: Some(Vec2::new(length, 3.0)),
                        ..default()
                    },
                    Transform {
                        translation: midpt.extend(2.0),
                        rotation: Quat::from_rotation_z(angle),
                        ..default()
                    },
                    Line,
                ));
            }
        }
    }
}

pub fn spawn_health_bars(
    mut commands: Commands,
    query: Query<(Entity, &Health, Option<&SpriteSize>, Option<&Sprite>), Without<HealthBar>>,
    mut health_bar_query: Query<(&HealthBarParent, &mut Sprite), With<HealthBar>>,
) {
    for (entity, health, maybe_size, maybe_sprite) in query.iter() {
        let health_ratio = health.current as f32 / health.max as f32;
        if health_ratio >= 1.0 {
            continue;
        }

        let parent_height = match (maybe_size, maybe_sprite) {
            (Some(SpriteSize(size)), _) => size.y,
            (_, Some(sprite)) => sprite.custom_size.unwrap_or(Vec2::new(0.0, 0.0)).y,
            _ => 0.0
        };

        let y_offset = parent_height * 0.7;

        commands.entity(entity).insert(GlobalTransform::default());
        //commands.entity(entity).insert(InheritedVisibility::default()); //SEEMS BROKEN, makes stuff dissapear when hp lowers a single point. idk
        commands.entity(entity).with_children(|parent| {
            parent.spawn(HealthBarBundle {
                sprite: Sprite {
                    color: Color::linear_rgb(1.5 - (health_ratio * 1.5), (health_ratio - 0.5).max(0.0), 0.0),
                    custom_size: Some(Vec2::new(y_offset * health_ratio, 6.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, y_offset * 0.9, 0.0),
                health_bar: HealthBar,
                health_bar_parent: HealthBarParent(),
            });
        });
    }
    for (_, mut sprite) in health_bar_query.iter_mut() {
        sprite.custom_size = None;
    }
}

fn spawn_enemies (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_time: Res<GameTimer>,
    game_state: Res<GameState>,
    enemies: Query<&Enemy>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    if game_time.timer.round() % ENEMY_SPAWNS == ENEMY_SPAWNS - 1.0 {
        for enemy in enemies.iter() {
            if ((GAME_TIME - game_time.timer) / ENEMY_SPAWNS).round() as usize == enemy.id {
                return;
            }
        }
        //println!("game_time.timer / ENEMY_SPAWNS = {}", game_time.timer / ENEMY_SPAWNS);
        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::new(90.0, 90.0)),
                image: asset_server.load("enemy.png"),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 15.0),
            Enemy {id: ((GAME_TIME - game_time.timer) / ENEMY_SPAWNS).round() as usize, memory: Vec::new(), memory_recency: true, target: None, rage: true, stun_timer: 0.0},
            Health { current: ENEMY_CURRENT_HP, max: ENEMY_MAX_HP},
        ));
    }
}
