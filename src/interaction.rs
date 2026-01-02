use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use std::process::exit;
use std::collections::BTreeMap;

use crate::stats::GameState;
use crate::stats::States;
use crate::stats::GameTimer;
use crate::stats::Health;
use crate::stats::CursorPos;
use crate::stats::Player;
use crate::stats::MenuScreen;
use crate::stats::QuitButton;
use crate::stats::PlayButton;
use crate::stats::Grid;
use crate::stats::Box;
use crate::stats::Map;
use crate::stats::Enemy;
use crate::stats::SPAWNABLE_RANGE;
//use crate::stats::Line;
use crate::stats::SPEED;
use crate::stats::GAME_TIME;
use crate::stats::PLAY_BUTTON_POS;
use crate::stats::QUIT_BUTTON_POS;
use crate::stats::PLAYER_BUILD_STUN;


pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            track_cursor,
            character_movement,
            quit_button,
            play_button,
            spawn_object_at_cursor,
        ));
        app.insert_resource(CursorPos(Vec2::default(), None));
        app.insert_resource(GameState { game_state: States::StartMenu, update_pathfind: false, grid_pairs: BTreeMap::new()});
        app.insert_resource(GameTimer { timer: GAME_TIME });
    }
}

fn track_cursor(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for event in cursor_moved_events.read() {
        cursor_pos.0 = event.position;
    }

    if let Ok(window) = window_query.single() {
        if let Ok((camera, camera_transform)) = camera_query.single() {
            if let Some(world_position) = window.cursor_position()
                .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
                .map(|ray| ray.unwrap().origin.truncate())
            {
                cursor_pos.1 = Some(world_position);
            }
        }
    }
}

fn character_movement(
    mut player: Query<(&mut Transform, &mut Player), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    if let Ok((mut transform, mut player_component)) = player.single_mut() { 
        if player_component.stun_timer > 0.0 {
            player_component.stun_timer -= time.delta_secs();
        }
        else {
            let speed: f32 = SPEED * time.delta_secs();
            if input.pressed(KeyCode::KeyA) && !input.pressed(KeyCode::KeyD) {
                let mut boxes_checked = true;
                for (k, v) in &game_state.grid_pairs {
                    if *v {
                        if transform.translation.x >= (k.0 - 35) as f32 && transform.translation.x <= (k.0 + 41) as f32 
                        && transform.translation.y >= (k.1 - 35) as f32 && transform.translation.y <= (k.1 + 35) as f32 {
                            transform.translation.x = (k.0 + 41) as f32;
                            boxes_checked = false;
                        }
                    }
                }
                if boxes_checked {
                    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::KeyS) {
                        transform.translation.x -= speed / 1.4;
                    }
                    else {
                        transform.translation.x -= speed;
                    }
                }
            }
            if input.pressed(KeyCode::KeyD) && !input.pressed(KeyCode::KeyA) {
                let mut boxes_checked = true;
                for (k, v) in &game_state.grid_pairs {
                    if *v {
                        if transform.translation.x >= (k.0 - 41) as f32 && transform.translation.x <= (k.0 + 35) as f32 
                        && transform.translation.y >= (k.1 - 35) as f32 && transform.translation.y <= (k.1 + 35) as f32 {
                            transform.translation.x = (k.0 - 41) as f32;
                            boxes_checked = false;
                        }
                    }
                }
                if boxes_checked {
                    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::KeyS) {
                        transform.translation.x += speed / 1.4;
                    }
                    else {
                        transform.translation.x += speed;
                    }
                }
            }
            if input.pressed(KeyCode::KeyW) && !input.pressed(KeyCode::KeyS) {
                let mut boxes_checked = true;
                for (k, v) in &game_state.grid_pairs {
                    if *v {
                        if transform.translation.x >= (k.0 - 35) as f32 && transform.translation.x <= (k.0 + 35) as f32 
                        && transform.translation.y >= (k.1 - 41) as f32 && transform.translation.y <= (k.1 + 35) as f32 {
                            transform.translation.y = (k.1 - 41) as f32;
                            boxes_checked = false;
                        }
                    }
                }
                if boxes_checked {
                    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::KeyD) {
                        transform.translation.y += speed / 1.4;
                    }
                    else {
                        transform.translation.y += speed;
                    }
                }
            }
            if input.pressed(KeyCode::KeyS) && !input.pressed(KeyCode::KeyW) {
                let mut boxes_checked = true;
                for (k, v) in &game_state.grid_pairs {
                    if *v {
                        if transform.translation.x >= (k.0 - 35) as f32 && transform.translation.x <= (k.0 + 35) as f32 
                        && transform.translation.y >= (k.1 - 35) as f32 && transform.translation.y <= (k.1 + 41) as f32 {
                            transform.translation.y = (k.1 + 41) as f32;
                            boxes_checked = false;
                        }
                    }
                }
                if boxes_checked {
                    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::KeyD) {
                        transform.translation.y -= speed / 1.4;
                    }
                    else {
                        transform.translation.y -= speed;
                    }
                }
            }
            if input.pressed(KeyCode::Escape) {
                exit(1); //ez out
            }
        }
    }
}

fn play_button(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu: Query<Entity, With<MenuScreen>>,
    play_button: Query<Entity, With<PlayButton>>,
    quit_button: Query<Entity, With<QuitButton>>,
    player: Query<Entity, With<Player>>,
    mut game_state: ResMut<GameState>,
) {
    if let Ok(_) = menu.single() {
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Some(world_position) = cursor_pos.1 {
                if world_position.x > PLAY_BUTTON_POS.2 - (PLAY_BUTTON_POS.0 / 2.0) + 4.0 && world_position.x < PLAY_BUTTON_POS.2 + (PLAY_BUTTON_POS.0 / 2.0) - 4.0 {
                    if world_position.y > PLAY_BUTTON_POS.3 - (PLAY_BUTTON_POS.1 / 2.0) + 4.0 && world_position.y < PLAY_BUTTON_POS.3 + (PLAY_BUTTON_POS.1 / 2.0) - 4.0 {
                        commands.entity(menu.single().unwrap()).despawn();
                        commands.entity(play_button.single().unwrap()).despawn();
                        commands.entity(quit_button.single().unwrap()).despawn();
                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(1955.0, 1955.0)),
                                color: Color::linear_rgba(0.6, 0.4, 1.0, 1.0),
                                image: asset_server.load("Level1.png"),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 0.0),
                            Map,
                        )); 
                        
                        commands.entity(player.single().unwrap()).insert(Visibility::Visible);
                        let circle = commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(SPAWNABLE_RANGE * 2.0, SPAWNABLE_RANGE * 2.0)),
                                color: Color::linear_rgba(1.0, 1.0, 1.0, 0.15),
                                image: asset_server.load("circle.png"),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 0.0, 1.0),
                        )).id();
                        commands.entity(player.single().unwrap()).add_child(circle);
                        
                        game_state.game_state = States::Level1;
                    }
                }
            }
        }
    }
}

fn quit_button(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPos>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(world_position) = cursor_pos.1 {
            if world_position.x > QUIT_BUTTON_POS.2 - (QUIT_BUTTON_POS.0 / 2.0) + 4.0 && world_position.x < QUIT_BUTTON_POS.2 + (QUIT_BUTTON_POS.0 / 2.0) - 4.0 {
                if world_position.y > QUIT_BUTTON_POS.3 - (QUIT_BUTTON_POS.1 / 2.0) + 4.0 && world_position.y < QUIT_BUTTON_POS.3 + (QUIT_BUTTON_POS.1 / 2.0) - 4.0 {
                    std::process::exit(0);
                }
            }
        }
    }
}

fn spawn_object_at_cursor(
    //mut enemy_pos: Query<(&mut Transform, &mut Sprite, &mut Enemy), (With<Enemy>, Without<Player>)>,
    mut player: Query<(&mut Transform, &mut Player), With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    mut grid: Query<(&Transform, &mut Sprite), (With<Grid>, Without<Player>, Without<Enemy>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    //for (mut transform, mut sprite, mut enemy_component) in &mut enemy_pos {
    if let Ok((mut player_transform, mut player_component)) = player.single_mut() {
        let player_pos = player_transform.translation;

        if let Some(world_position) = cursor_pos.1 {
            let mut wx: i32 = (world_position.x as i32 / 50) * 50;
            let mut wy: i32 = (world_position.y as i32 / 50) * 50;
            if world_position.x >= 0.0 { wx += 25;}
            else { wx -= 25;}
            if world_position.y >= 0.0 { wy += 25;}
            else { wy -= 25;}

            for (&cell_transform, mut cell_sprite) in &mut grid {
                if wx as f32 >= cell_transform.translation.x - 25.0 && 
                    wx as f32 <= cell_transform.translation.x + 25.0 && 
                    wy as f32 >= cell_transform.translation.y - 25.0 && 
                    wy as f32 <= cell_transform.translation.y + 25.0 {
                    cell_sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, 0.6);
                }
                else {
                    cell_sprite.color = Color::linear_rgba(1.0, 1.0, 1.0, 0.2);
                }
            }

            if keyboard_input.just_pressed(KeyCode::Space) { 
                if player_component.stun_timer > 0.0 { return; }
                if (Vec2::new(player_pos.x, player_pos.y) - Vec2::new(world_position.x, world_position.y).trunc()).length() > SPAWNABLE_RANGE {
                    let delta = Vec2::new(player_pos.x, player_pos.y) - world_position;
                    let angle = delta.y.atan2(delta.x);
                    let circle = Vec2::new(player_pos.x, player_pos.y) + Vec2::new(angle.cos(), angle.sin()) * SPAWNABLE_RANGE * -1.0;
                    wx = (circle.x as i32 / 50) * 50;
                    wy = (circle.y as i32 / 50) * 50;
                    if circle.x >= 0.0 { wx += 25;}
                    else { wx -= 25;}
                    if circle.y >= 0.0 { wy += 25;}
                    else { wy -= 25;}
                }
                if (Vec2::new(player_pos.x, player_pos.y) - Vec2::new(wx as f32, wy as f32).trunc()).length() > 85.0 {
                    let mut boxes_checked = true;

                    if game_state.grid_pairs[&(wx-25, wy-25)] 
                    || game_state.grid_pairs[&(wx-25, wy+25)] 
                    || game_state.grid_pairs[&(wx+25, wy-25)] 
                    || game_state.grid_pairs[&(wx+25, wy+25)] {
                        boxes_checked = false;
                    }
                    if boxes_checked {
                        commands.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(90.0, 90.0)),
                                image: asset_server.load("box.png"),
                                ..default()
                            },
                            Transform::from_xyz(wx as f32, wy as f32, 10.0),
                            Box,
                            Health { current: 2, max: 2},
                        ));
                        //game_state.boxes.push(((wx, wy), (50, 50)));
                        game_state.grid_pairs.entry((wx-25, wy-25)).or_default();
                        *game_state.grid_pairs.get_mut(&(wx-25, wy-25)).unwrap() = true;
                        game_state.grid_pairs.entry((wx+25, wy-25)).or_default();
                        *game_state.grid_pairs.get_mut(&(wx+25, wy-25)).unwrap() = true;
                        game_state.grid_pairs.entry((wx-25, wy+25)).or_default();
                        *game_state.grid_pairs.get_mut(&(wx-25, wy+25)).unwrap() = true;
                        game_state.grid_pairs.entry((wx+25, wy+25)).or_default();
                        *game_state.grid_pairs.get_mut(&(wx+25, wy+25)).unwrap() = true;
                        
                        game_state.update_pathfind = true;

                        //add stun next
                        player_component.stun_timer = PLAYER_BUILD_STUN;
                    }
                }
            }
        }
    }
}

fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}