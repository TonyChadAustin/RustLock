use bevy::prelude::*;
//use bevy::reflect::List;
use std::collections::{HashMap, HashSet, VecDeque};
use std::cmp::Ordering;

//use crate::stats::GameStats;
use crate::stats::{Player,
    QuitButton,
    SettingsButton,
    PlayButton,
    MenuScreen,
    Grid,
    GameState,
    States,
    GameTimer,
    Health,
    Box,
    Enemy,
    PathfindTrigger,
    GRID_SIZE,
    ENEMY_ATTACK_STUN,
    PLAY_BUTTON_POS,
    SETTINGS_BUTTON_POS,
    QUIT_BUTTON_POS,
    TimerText,
    PLAYER_HP,
};


pub struct Node {
    x: i32,
    y: i32,
    distance: i32,
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.distance == other.distance
    }
}
impl Eq for Node{}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance)
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (
            spawn_grid,
            despawn_boxes,
            enemy_movement,
            pathfinding,
        ));
        app.insert_resource(PathfindTrigger { triggered: false });
    }
}


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn( (
        Sprite {
            custom_size: Some(Vec2::new(2555.0, 1435.0)),
            image: asset_server.load("menu.png"),
            ..default()
        },
        Transform::from_xyz(300.0, 0.0, 0.0),
        MenuScreen,
    ));
    commands.spawn( (
        Sprite {
            custom_size: Some(Vec2::new(PLAY_BUTTON_POS.0, PLAY_BUTTON_POS.1)),
            image: asset_server.load("play.png"),
            ..default()
        },
        Transform::from_xyz(PLAY_BUTTON_POS.2, PLAY_BUTTON_POS.3, 1.0),
        PlayButton,
    ));
    commands.spawn( (
        Sprite {
            custom_size: Some(Vec2::new(SETTINGS_BUTTON_POS.0, SETTINGS_BUTTON_POS.1)),
            image: asset_server.load("settings.png"),
            ..default()
        },
        Transform::from_xyz(SETTINGS_BUTTON_POS.2, SETTINGS_BUTTON_POS.3, 1.0),
        SettingsButton,
    ));
    commands.spawn( (
        Sprite {
            custom_size: Some(Vec2::new(QUIT_BUTTON_POS.0, QUIT_BUTTON_POS.1)),
            image: asset_server.load("quit.png"),
            ..default()
        },
        Transform::from_xyz(QUIT_BUTTON_POS.2, QUIT_BUTTON_POS.3, 1.0),
        QuitButton,
    ));
    let player = commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(40.0, 40.0)),
            image: asset_server.load("animerppng.png"),
            ..default()
        },
        Transform::from_xyz(300.0, 0.0, 999.0),
        Player {stun_timer: 0.0},
        Health {current: PLAYER_HP, max: PLAYER_HP},
        Visibility::Hidden,
    )).id();

    //commands.entity(player).with_children(|parent| {
     //   parent.spawn(Camera2d::default());
    //});

    commands.entity(player).with_children(|parent| {
        let camera = parent.spawn(Camera2d::default()).id();
        
        parent.commands().entity(camera).with_children(|cam| {
            cam.spawn((
                Text2d::new("30"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 600.0, 10.0),
                TimerText,
            ));
        });
    });
}

fn spawn_grid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player: Query<&Transform, With<Player>>,
    mut grids: Query<(Entity, &mut Grid)>,
    mut game_state: ResMut<GameState>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }

    let player_transform = player.single().unwrap();
    let mut added_this_frame: Vec<(f32, f32)> = Vec::new();
    for (grid_entity, grid) in &mut grids { //cleanup grid that is far away
        if (grid.xy.x - player_transform.translation.x).abs() > GRID_SIZE as f32 || (grid.xy.y - player_transform.translation.y).abs() > GRID_SIZE as f32 {
            commands.entity(grid_entity).despawn(); // This causes lag on big maps
            //commands.entity(grid_entity).insert(Visibility::Hidden);
        }
    }   
    for y in 0..(GRID_SIZE / 25) + 1 {
        for x in 0..(GRID_SIZE / 25) + 1 {
            let normalized_x = (((player_transform.translation.x) + (x as f32 - (GRID_SIZE / 50) as f32) * 50.0) as i32 / 50) as f32 * 50.0;
            let normalized_y = (((player_transform.translation.y) + (y as f32 - (GRID_SIZE / 50) as f32) * 50.0) as i32 / 50) as f32 * 50.0;
            let mut check_existing = false;
            for (_, grid) in &mut grids {
                if (grid.xy.x - normalized_x).abs() < 10.0 && (grid.xy.y - normalized_y).abs() < 10.0 {
                    check_existing = true;
                }
            }
            for (x, y) in &added_this_frame {
                if (x - normalized_x).abs() < 10.0 && (y - normalized_y).abs() < 10.0 {
                    check_existing = true;
                }
            }
            if !check_existing {
                if normalized_x.abs() < 1000.0 && normalized_y.abs() < 1000.0 {
                    commands.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(50.0, 50.0)),
                            image: asset_server.load("cell.png"),
                            color: Color::linear_rgba(1.0, 1.0, 1.0, 0.2),
                            ..default()
                        },
                        Transform::from_xyz(normalized_x, normalized_y, 5.0),
                        Grid {xy: Vec2::new(normalized_x, normalized_y)},
                        Visibility::Visible,
                    ));
                    if !game_state.grid_pairs.contains_key(&(normalized_x as i32, normalized_y as i32)) { 
                        game_state.grid_pairs.insert((normalized_x as i32, normalized_y as i32), false);
                    }
                    added_this_frame.push((normalized_x, normalized_y));
                }
                else {
                    commands.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(50.0, 50.0)),
                            color: Color::BLACK,
                            ..default()
                        },
                        Transform::from_xyz(normalized_x, normalized_y, 5555.0),
                        Grid {xy: Vec2::new(normalized_x, normalized_y)},
                        Visibility::Visible,
                    ));
                    if !game_state.grid_pairs.contains_key(&(normalized_x as i32, normalized_y as i32)) { 
                        game_state.grid_pairs.insert((normalized_x as i32, normalized_y as i32), true);
                    }
                    added_this_frame.push((normalized_x, normalized_y));
                }
            }
        }
    }
}

pub fn despawn_boxes(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    boxes: Query<(Entity, &Health, &Transform), With<Box>>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    for (entity, health, transform) in boxes.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn();
            let wx = transform.translation.x as i32;
            let wy = transform.translation.y as i32;
            game_state.grid_pairs.entry((wx as i32, wy as i32)).or_insert(false);
            //game_state.boxes.retain(|((x,y),(_,_))| *x != transform.translation.x as i32 || *y != transform.translation.y as i32);
            
            game_state.grid_pairs.entry((wx-25, wy-25)).or_default();
            *game_state.grid_pairs.get_mut(&(wx-25, wy-25)).unwrap() = false;
            game_state.grid_pairs.entry((wx+25, wy-25)).or_default();
            *game_state.grid_pairs.get_mut(&(wx+25, wy-25)).unwrap() = false;
            game_state.grid_pairs.entry((wx-25, wy+25)).or_default();
            *game_state.grid_pairs.get_mut(&(wx-25, wy+25)).unwrap() = false;
            game_state.grid_pairs.entry((wx+25, wy+25)).or_default();
            *game_state.grid_pairs.get_mut(&(wx+25, wy+25)).unwrap() = false;
        }
    }
}

fn enemy_movement (
    mut enemy_pos: Query<(&mut Transform, &mut Sprite, &mut Enemy), (With<Enemy>, Without<Player>)>,
    mut player: Query<(&Transform, &mut Health), With<Player>>,
    mut boxes: Query<(&mut Transform, &mut Health), (With<Box>, Without<Enemy>, Without<Player>)>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    game_state: ResMut<GameState>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    let attack_stun_time = ENEMY_ATTACK_STUN;
    for (mut transform, mut sprite, mut enemy_component) in &mut enemy_pos {
        let speed: f32 = enemy_component.variant.speed() * time.delta_secs();
        
        if let Ok((player_transform, mut player_health)) = player.single_mut() {
            if enemy_component.stun_timer > 0.0 {
                enemy_component.rage = true;
                enemy_component.stun_timer -= time.delta_secs();
                continue;
            }
            if enemy_component.rage {
                sprite.image = asset_server.load("enemy_rage.png");
            }
            else {
                sprite.image = asset_server.load(enemy_component.image);
            }
            
            let player_pos = player_transform.translation.truncate();

            if enemy_component.rage {
                enemy_component.memory.clear();
                if (transform.translation.x - player_pos.x).abs() < 31.0 && (transform.translation.y - player_pos.y).abs() < 31.0 {
                    if player_health.current > 0 {
                        player_health.current -= enemy_component.variant.damage();
                        enemy_component.stun_timer = attack_stun_time;
                    }
                }
                else {
                    //Move forward towards the player until reaching a box that blocks the path.
                    let target_pos = Vec3::new(player_pos.x, player_pos.y, 0.0);
                    let direction = (target_pos - transform.translation).truncate().normalize();
                    let next_pos = transform.translation + (direction * speed).extend(0.0);
                    let mut wx = (next_pos.x as i32 / 50) * 50;
                    let mut wy = (next_pos.y as i32 / 50) * 50;
                    if next_pos.x >= 0.0 { wx += 25;}
                    else { wx -= 25;}
                    if next_pos.y >= 0.0 { wy += 25;}
                    else { wy -= 25;}
                    
                    let mut blocked: (bool, Vec2) = (false, Vec2::new(0.0, 0.0));
                    if game_state.grid_pairs[&(wx as i32-25, wy as i32-25)] 
                    || game_state.grid_pairs[&(wx as i32-25, wy as i32+25)] 
                    || game_state.grid_pairs[&(wx as i32+25, wy as i32-25)] 
                    || game_state.grid_pairs[&(wx as i32+25, wy as i32+25)] {
                        blocked = (true, Vec2::new(wx as f32, wy as f32));
                    }
                    if blocked.0 {
                        for (box_transform, mut box_health) in &mut boxes {
                            if (box_transform.translation.x - blocked.1.x).abs() < 90.0 && (box_transform.translation.y - blocked.1.y).abs() < 90.0 {
                                if box_health.current > 0 {
                                    box_health.current -= enemy_component.variant.damage();
                                    enemy_component.stun_timer = attack_stun_time;
                                }
                            }
                        }
                    }
                    else {
                        transform.translation = next_pos;
                    }
                    //Stop moving, face the center of that box and attack the box until it breaks or new enemy_component.memory is found.
                    //On attack, set stun timer to attack stun time and reduce that box's health.
                }
            }
            else {
                //TODO: Simplify the enemy memory to just each significant turn, then let them casually path find to it like checkpoints. This way being blocked by other enemies can work.
                /*
                PATH SMOOTHING: Loop through the path and identify points that change directions.
                Direction changes equal 'optimal points'. The best path goes through optimal points.
                Search between each set of optimal points with increments of 100 units like a ray-trace.
                If the ray doesn't hit a box, then remove all path steps between the two optimal points.
                ///HERE */
                if enemy_component.memory.len() > 5 {
                    let mut furthest: usize = 0;
                    let mut blocked = false;
                    //println!("Memory: {:?} ", enemy_component.memory);
                    for n in 1..enemy_component.memory.len() - 1 {
                        //raytrace from enemy_component.memory[0] to each node until fail.
                        let checks = (Vec2::new(enemy_component.memory[0].0 as f32, enemy_component.memory[0].1 as f32)
                        - Vec2::new(enemy_component.memory[n].0 as f32, enemy_component.memory[n].1 as f32)).length() / 25.0;
                        let direction = (Vec2::new(enemy_component.memory[n].0 as f32, enemy_component.memory[n].1 as f32) 
                        - Vec2::new(enemy_component.memory[0].0 as f32, enemy_component.memory[0].1 as f32)).normalize();
                        //println!("checks: {}, direction: {:?}", checks, direction);
                        for c in 0..checks as i32+ 1 as i32 {
                            let check_spot = Vec2::new(
                                enemy_component.memory[0].0 as f32 + (25.0 * (c) as f32) * direction.x,
                                enemy_component.memory[0].1 as f32 + (25.0 * (c) as f32) * direction.y,
                            );
                            let fifty = (((check_spot.x as i32 / 50) * 50), ((check_spot.y as i32 / 50) * 50));
                            let twentyfive = (((check_spot.x as i32 / 25) * 25), ((check_spot.y as i32 / 25) * 25));
                            //println!("fifty: {:?}, twentyfive: {:?} ", fifty, twentyfive);
                            if fifty != twentyfive {
                                if game_state.grid_pairs.contains_key(&(twentyfive.0-25, twentyfive.1-25))
                                && game_state.grid_pairs.contains_key(&(twentyfive.0-25, twentyfive.1+25))
                                && game_state.grid_pairs.contains_key(&(twentyfive.0+25, twentyfive.1-25))
                                && game_state.grid_pairs.contains_key(&(twentyfive.0+25, twentyfive.1+25)) {
                                    if game_state.grid_pairs[&(twentyfive.0-25, twentyfive.1-25)] 
                                    || game_state.grid_pairs[&(twentyfive.0-25, twentyfive.1+25)]
                                    || game_state.grid_pairs[&(twentyfive.0+25, twentyfive.1-25)]
                                    || game_state.grid_pairs[&(twentyfive.0+25, twentyfive.1+25)]{
                                        blocked = true;
                                        break;
                                    }
                                }
                            }
                            else {
                                for i in -1..2 {
                                    for j in -1..2 {
                                        if game_state.grid_pairs.contains_key(&(twentyfive.0+(50*i), twentyfive.1+(50*j))) {
                                            if game_state.grid_pairs[&(twentyfive.0+(50*i), twentyfive.1+(50*j))] {
                                                blocked = true;
                                                break;
                                            }
                                        }
                                    }
                                    if blocked { break; }
                                }
                                if blocked { break; }
                            }
                        }
                        if blocked { break; }
                        else {
                            //println!("Furthest updated to: {:?} ", enemy_component.memory[n]);
                            furthest = n;
                        }
                    }
                    if furthest > 1 {
                        for _ in 0..furthest-2 {
                            enemy_component.memory.remove(1);
                        }
                    } 
                }



                if (transform.translation.x - player_pos.x).abs() < 31.0 && (transform.translation.y - player_pos.y).abs() < 31.0 {
                    enemy_component.rage = true;
                }
                else {
                    if enemy_component.memory.len() > 1 {
                        let target_pos = Vec2::new(enemy_component.memory[1].0 as f32, enemy_component.memory[1].1 as f32);
                        if (transform.translation.truncate() - Vec2::new(target_pos.x, target_pos.y)).length() < 5.0 {
                            enemy_component.memory.remove(0);
                        }
                        let direction = (Vec2::new(target_pos.x as f32, target_pos.y as f32) - Vec2::new(transform.translation.x, transform.translation.y)).normalize();
                        transform.translation += (direction * speed).extend(0.0);
                    }
                    else {
                        if let Some(target) = enemy_component.target {
                            let direction = (Vec2::new(target.0 as f32, target.1 as f32) - Vec2::new(transform.translation.x, transform.translation.y)).normalize();
                            transform.translation += (direction * speed).extend(0.0);
                        }
                    }
                }
            }
        }
    }
}



fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn pathfinding(
    timer: Res<GameTimer>,
    mut game_state: ResMut<GameState>,
    player: Query<&Transform, With<Player>>,
    grid: Query<&Transform, With<Grid>>,
    mut trigger: ResMut<PathfindTrigger>,
    mut enemies: Query<(&mut Enemy, &Transform), With<Enemy>>,
) {
    match game_state.game_state {
        States::Level1 => (),
        _ => return,
    }
    if grid.is_empty() { return; }

    if !game_state.update_pathfind {
        if (timer.timer * 10.0) as i32 % 8 != 0 {
            trigger.triggered = false;
        }
        if trigger.triggered == true { return; }
    }
    trigger.triggered = true;
    game_state.update_pathfind = false;

    if let Ok(player_transform) = player.single() {
        let mut enemy_list: Vec<usize> = Vec::new();
        for (mut enemy, _enemy_transform) in enemies.iter_mut() {
            enemy.memory_recency = false;
            //enemy.memory.clear();
            enemy_list.push(enemy.id);
        }

        let player_pos = (player_transform.translation.x as i32, player_transform.translation.y as i32);
        let mut open_cells = HashSet::new();
        let mut closed_cells = HashSet::new();
        let mut parents = HashMap::new();

        /*let mut count = 1;
        while open_cells.is_empty() {
            for cell in grid.iter() { //this can be way more efficient
                let cell_pos = (cell.translation.x as i32, cell.translation.y as i32);
                if manhattan_distance(cell_pos, player_pos) > 100 * count {
                    //closed_cells.insert(cell_pos);
                    continue;
                }

                let mut wx = (cell_pos.0 as i32 / 50) * 50;
                let mut wy = (cell_pos.1 as i32 / 50) * 50;
                if cell_pos.0 >= 0 { wx += 25;}
                else { wx -= 25;}
                if cell_pos.1 >= 0 { wy += 25;}
                else { wy -= 25;}
                let mut box_check = true;
                if game_state.grid_pairs[&(wx-25, wy-25)] 
                || game_state.grid_pairs[&(wx-25, wy+25)] 
                || game_state.grid_pairs[&(wx+25, wy-25)] 
                || game_state.grid_pairs[&(wx+25, wy+25)] {
                    closed_cells.insert((wx, wy));
                    box_check = false;
                }

                if !box_check {
                    continue;
                }
                open_cells.insert((wx, wy));
            }
            count += 1;
        } */
        let mut wx = (player_pos.0 as i32 / 50) * 50;
        let mut wy = (player_pos.1 as i32 / 50) * 50;
        if player_pos.0 >= 0 { wx += 25;}
        else { wx -= 25;}
        if player_pos.1 >= 0 { wy += 25;}
        else { wy -= 25;}
        for i in -2..3 {
            for j in -2..3 {
                if game_state.grid_pairs.contains_key(&(wx+(50*i)-25, wy+(50*j)-25))
                && game_state.grid_pairs.contains_key(&(wx+(50*i)-25, wy+(50*j)+25))
                && game_state.grid_pairs.contains_key(&(wx+(50*i)+25, wy+(50*j)-25))
                && game_state.grid_pairs.contains_key(&(wx+(50*i)+25, wy+(50*j)+25)) {
                    if !game_state.grid_pairs[&(wx+(50*i)-25, wy+(50*j)-25)] 
                    && !game_state.grid_pairs[&(wx+(50*i)-25, wy+(50*j)+25)]
                    && !game_state.grid_pairs[&(wx+(50*i)+25, wy+(50*j)-25)]
                    && !game_state.grid_pairs[&(wx+(50*i)+25, wy+(50*j)+25)]{
                        open_cells.insert((wx + 50 * i, wy + 50 * j));
                    }
                }
            }
        }
        if open_cells.is_empty() { 
            for &enemy_id in &enemy_list {
                for (mut enemy, _enemy_transform) in enemies.iter_mut() {
                    if enemy.id == enemy_id {
                        enemy.memory.clear();
                        enemy.rage = true;
                        enemy.memory_recency = false;
                    }
                }
            }
            return; 
        }

        let mut open: Vec<_> = open_cells.iter().collect();
        open.sort_by(|a, b| {
            let a_dist = manhattan_distance(**a, player_pos);
            let b_dist = manhattan_distance(**b, player_pos);
            a_dist.cmp(&b_dist)
        });
        //let mut wx = (open[0].0 as i32 / 50) * 50;
        //let mut wy = (open[0].1 as i32 / 50) * 50;
        //if open[0].0 >= 0 { wx += 25;}
        //else { wx -= 25;}
        //if open[0].1 >= 0 { wy += 25;}
        //else { wy -= 25;}
        let target_ret = *open[0];
        let mut open_cells = VecDeque::new();
        open_cells.push_back(*open[0]);
        parents.insert(*open[0], *open[0]);
        closed_cells.clear();
        //println!("Player: {:?}", player_pos);
        //println!("Closest cell: {:?}", open_cells[0]);

        let max_iterations = 5000;
        let mut iterations = 0;

        while let Some(current) = open_cells.pop_front() {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            for (mut enemy, enemy_transform) in enemies.iter_mut() {
                if enemy.memory_recency == true { continue; }
                enemy.target = Some(target_ret);
                
                if (enemy_transform.translation.x as i32) > current.0 - 50 &&
                 (enemy_transform.translation.x as i32) < current.0 + 50 &&
                 (enemy_transform.translation.y as i32) > current.1 - 50 &&
                 (enemy_transform.translation.y as i32) < current.1 + 50 
                 {
                    let mut path = vec![];
                    let mut c = current;

                    while c != target_ret {
                        path.push(c);
                        c = parents[&c];
                    }
                    path.push(target_ret);
                    for enemy_id in enemy_list.clone() {
                        if enemy.id == enemy_id {
                            enemy_list.retain(|&x| x != enemy_id);
                        }
                    }
                    if path.len() > 1 {
                        enemy.rage = false;
                    }
                    enemy.memory = path;
                    enemy.memory_recency = true;

                }
            }
            
            closed_cells.insert(current);
            
            let neighbors = [
                (current.0 - 50, current.1), 
                (current.0 + 50, current.1), 
                (current.0, current.1 - 50), 
                (current.0, current.1 + 50), 
            ];
            for &neighbor in &neighbors {
                if closed_cells.contains(&neighbor) {
                    continue;
                }
                if open_cells.contains(&neighbor) {
                    continue;
                }

                if game_state.grid_pairs.contains_key(&(neighbor.0-25, neighbor.1-25)) 
                && game_state.grid_pairs.contains_key(&(neighbor.0-25, neighbor.1+25))
                && game_state.grid_pairs.contains_key(&(neighbor.0+25, neighbor.1-25))
                && game_state.grid_pairs.contains_key(&(neighbor.0+25, neighbor.1+25)) {
                    if game_state.grid_pairs[&(neighbor.0-25, neighbor.1-25)] 
                    || game_state.grid_pairs[&(neighbor.0-25, neighbor.1+25)]
                    || game_state.grid_pairs[&(neighbor.0+25, neighbor.1-25)] 
                    || game_state.grid_pairs[&(neighbor.0+25, neighbor.1+25)] {
                        continue;
                    }
                }
                else {
                    //break; return;
                }
                

                parents.insert(neighbor, current);
                open_cells.push_back(neighbor);
            }
        }
        for &enemy_id in &enemy_list {
            for (mut enemy, _enemy_transform) in enemies.iter_mut() {
                if enemy.id == enemy_id {
                    enemy.memory.clear();
                    enemy.rage = true;
                    enemy.memory_recency = false;
                }
            }
        }
    }
}

