mod render;
mod agents;
mod grid;
mod animation;
mod input;
mod resources;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::sync::{mpsc, Arc, LockResult, Mutex, RwLock};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use image::{open, Frame};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use minifb::Key::{K, R};
use rand::{random, random_range};
use crate::agents::{find_path, Agent, AgentCoroutine, AgentEvent, AgentTask};
use crate::agents::AgentEvent::AgentAddTask;
use crate::animation::{Animation, AnimationPool};
use crate::grid::{find_horizontal_neighbours, get_manhattan_distance, is_horizontal_neighbour, Cube, Grid, Light};
use crate::input::{InputBuffer, InputState};
use crate::render::{draw_face, draw_sprite, light_flood_fill, Face, Sprite};
use crate::resources::{load_animations, load_cube_sprites};

const SCREEN_WIDTH: usize = 2000;
const SCREEN_HEIGHT: usize = 1200;
const SCREEN_Y_OFFSET: usize = SCREEN_HEIGHT / 4;

const TILE_WIDTH: usize = 24;
const TILE_HALF_WIDTH: usize = TILE_WIDTH / 2;

const GRID_HEIGHT: usize = 60;
const GRID_WIDTH: usize = 120;

const VIEW_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;

const EMPTY_CUBE: u8 = 255;
const WATER_CUBE: u8 = 6;
const LANTERN_CUBE: u8 = 7;

const SUN_LIGHT: (u8, u8, u8) = (205, 205, 205);


fn advance_task(agent: &mut Agent, grid: Arc<Mutex<Grid>>) {
    let mut completed = false;
    if let Some(task_wrapper) = &mut agent.active_task {
        match &mut task_wrapper.task {
            AgentTask::Move { path, ..} => {
                let next = path.pop().unwrap();
                if path.is_empty() {
                    completed = true;
                }
                {
                    let grid_lock = grid.lock();
                    match grid_lock {
                        Ok(mut grid) => {
                            if !grid.is_occupied(next) {
                                let x_dir = next.0 as i32 - agent.position.0 as i32;
                                let y_dir = next.1 as i32 - agent.position.1 as i32;
                                if x_dir < 0 {
                                    agent.change_animation("running_nw");
                                }
                                else if x_dir > 0 {
                                    agent.change_animation("running_se");
                                }
                                else if y_dir < 0 {
                                    agent.change_animation("running_ne");
                                }
                                else {
                                    agent.change_animation("running_sw");
                                }
                                grid.move_cube(agent.position, next);
                                agent.position = next;

                            }
                            else {
                                if let Some(destination) = agent.destination {
                                    if let Some(path) = find_path(agent.position, destination, &*grid) {
                                        agent.tasks.insert(0, AgentTask::Move {
                                            path: path,
                                            destination: destination,
                                        });
                                    }
                                }

                                completed = true;
                            }

                        }
                        Err(_) => {}
                    }
                }
            }
            AgentTask::FindPath { destination } => {
                let start = agent.destination.or(Some(agent.position)).unwrap();
                {
                    let grid_lock = grid.lock();
                    if let Some(path) = find_path(start, *destination, grid_lock.unwrap().deref()) {
                        agent.tasks.insert(0, AgentTask::Move {
                            path: path,
                            destination: destination.clone(),
                        });
                    }

                    completed = true;
                }

            }
            AgentTask::Plough { target } => {
                if agent.position.2 > 0 {
                    if let Ok(mut grid) = grid.lock() {
                        let index = grid.get_vector_pos(*target).unwrap();
                        grid[index].cube_type = 4;
                        light_flood_fill((target.0, target.1, target.2), &mut *grid);
                    }
                }

                completed = true;
            }
            AgentTask::Dig { target } => {
                if agent.position.2 > 0 {
                    if let Ok(mut grid) = grid.lock() {
                        let index = grid.get_vector_pos(*target).unwrap();
                        grid[index].cube_type = EMPTY_CUBE;
                    }
                }
                completed = true;
            }
            AgentTask::Place { target, cube } => {
                if let Ok(mut grid) = grid.lock() {
                    grid.place_cube(*target, *cube);
                }
                completed = true;
            }
        }
    }
    if completed {
        if let Some(task_wrapper) = &mut agent.active_task {
            task_wrapper.completed = true;
        }
    }
}


fn main() {
    let sprites = load_cube_sprites();
    let grid = Arc::new(Mutex::new(prepare_grid()));
    let worker_animations = Arc::new(load_animations());
    let select_cube = Sprite::new("resources/24/select_cube.png");

    let man_x = GRID_WIDTH - 17;
    let man_y = GRID_WIDTH - 30;
    let man2_x = GRID_WIDTH - 17;
    let man2_y = GRID_WIDTH - 31;

    let (man_z, man2_z) = {
        let grid_lock = grid.lock().unwrap();
        (find_ground_spawn_z(&grid_lock, man_x, man_y), find_ground_spawn_z(&grid_lock, man2_x, man2_y))
    };

    let mut man = Agent {
        animation: worker_animations.animations["idle"].clone(),
        position: (man_x, man_y, man_z),
        animation_pool: Arc::clone(&worker_animations),
        name: "man".to_string(),
        animation_state: 0,
        destination: None,
        tasks: vec![],
        active_task: None,
        id: 0
    };

    let mut man2 = Agent {
        animation: worker_animations.animations["idle"].clone(),
        position: (man2_x, man2_y, man2_z),
        animation_pool: Arc::clone(&worker_animations),
        name: "man2".to_string(),
        animation_state: 0,
        destination: None,
        tasks: vec![],
        active_task: None,
        id: 1
    };

    grid.lock().unwrap().spawn_agent(&mut man);
    grid.lock().unwrap().spawn_agent(&mut man2);


    let (game_events, game_events_receiver) = mpsc::channel();
    let agents: Arc<Mutex<Vec<Agent>>> = Arc::new(Mutex::new(vec![man, man2]));
    let agent_clone = Arc::clone(&agents);
    let grid_clone = Arc::clone(&grid);

    let mut game_tick: u32 = 0;
    let agent_loop = thread::spawn(move || {
        loop {

            {
                //println!("game tick: {}", gate_tick);
                let mut mut_agents =  agent_clone.lock().unwrap();
                match game_events_receiver.try_recv() {
                    Ok(ev) => {
                        match ev {
                            AgentAddTask { agent, task } => {
                                println!("got task");
                                mut_agents[agent].tasks.push(task);
                            },
                            _ =>{}
                        }

                    },
                    Err(_) => {},
                }
                for mut agent in mut_agents.iter_mut() {
                    agent.advance_animation_state();
                    if let Some(active_task) = &agent.active_task {
                        //println!("active task: {:?}", active_task.end_tick);
                        if game_tick < active_task.end_tick {
                            continue;
                        }
                        //println!("gate tick: {}", gate_tick);
                        advance_task(agent, Arc::clone(&grid_clone));

                        if let Some(task) = &agent.active_task {
                            if !task.completed {
                               continue;
                            }
                            agent.active_task = None;
                        }
                    }

                    if agent.tasks.is_empty() {
                        agent.change_animation("idle");
                        continue;
                    }
                    let mut next_task = agent.tasks.remove(0);
                    let mut cancel_task = false;
                    if let Some(required_positions) = next_task.get_required_position() {
                        if !required_positions.contains(&agent.position) {
                            println!("not in required pos");
                            for pos in required_positions {
                                {
                                    let grid_lock = grid_clone.lock();
                                    if let Some(path) = find_path(agent.position, pos, grid_lock.unwrap().deref()) {
                                        agent.tasks.insert(0, next_task.clone());
                                        next_task = AgentTask::Move {
                                            destination: pos,
                                            path: path,
                                        };
                                        cancel_task = false;
                                        break;
                                    }
                                }
                                cancel_task = true;
                            }
                        }
                    }

                    let end_tick = match &next_task {
                        AgentTask::Plough {..} => {
                            agent.change_animation("ploughing");
                            game_tick + 15
                        },

                        AgentTask::Move { .. } => {
                            game_tick + 1
                        }
                        AgentTask::FindPath { .. } => {
                            game_tick
                        }
                        AgentTask::Dig { .. } => {
                            agent.change_animation("mining");
                            game_tick + 20
                        }
                        AgentTask::Place { .. } => {
                            game_tick + 9
                        }
                    };
                    let task = AgentCoroutine {
                        task: next_task,
                        end_tick,
                        completed: false,
                    };
                    agent.active_task = if cancel_task { None } else { Some(task) };

                }
            }
            sleep(Duration::from_millis(200));
            game_tick += 1;
        }
    });

    let mut window = Window::new(
        "Cubes",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("{}", e));

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = vec![0xFFFFFF; SCREEN_WIDTH * SCREEN_HEIGHT];


    let mut view_x = GRID_WIDTH - VIEW_WIDTH;
    let mut view_y = GRID_WIDTH - VIEW_WIDTH;
    let mut view_z = GRID_HEIGHT - VIEW_HEIGHT;
    let mut selected_cube: Option<(usize, usize, usize)> = None;
    let mut night_mode = false;
    let read_only_grid = Arc::clone(&grid);

    let mut input_buffer = InputBuffer::new();
    let mut selection_state = InputState {
        selected_agent: None
    };


    while window.is_open() && !window.is_key_down(Key::Escape) {
        let scroll_input = window.get_scroll_wheel().map(|scroll| {
            scroll.1
        });

        input_buffer.update_button_states(window.get_keys(), window.get_mouse_down(MouseButton::Left), window.get_mouse_down(MouseButton::Right));

        if let Some(scroll_input) = scroll_input {
            if scroll_input > 0.0 && view_z < GRID_HEIGHT - VIEW_HEIGHT {
                view_z += 1;
            }
            if scroll_input < 0.0 && view_z > 0 {
                view_z -= 1;
            }
        }

        let mut buffer = buffer.clone();
        if input_buffer.button_pressed(Key::N) {
            night_mode = !night_mode;
        }
        if input_buffer.button_pressed_or_held(Key::D) && view_x < GRID_WIDTH - VIEW_WIDTH {
            view_x += 1;
        }
        if  input_buffer.button_pressed_or_held(Key::A) && view_x > 0 {
            view_x -= 1;
        }
        if  input_buffer.button_pressed_or_held(Key::S) && view_y > 0 {
            view_y -= 1;
        }
        if  input_buffer.button_pressed_or_held(Key::W) && view_y < GRID_WIDTH - VIEW_WIDTH {
            view_y += 1;
        }

        if let Some(sel) = selected_cube {
            if  input_buffer.button_pressed(Key::Right) && sel.0 < VIEW_WIDTH {
                selected_cube = Some((sel.0 + 1, sel.1, sel.2));
            }
            if  input_buffer.button_pressed(Key::Left) && sel.0 > 0 {
                selected_cube = Some((sel.0 - 1, sel.1, sel.2));
            }
            if  input_buffer.button_pressed(Key::Down) && sel.1 < VIEW_WIDTH {
                selected_cube = Some((sel.0, sel.1 + 1, sel.2));
            }
            if  input_buffer.button_pressed(Key::Up) && sel.1 > 0 {
                selected_cube = Some((sel.0, sel.1 - 1, sel.2));
            }
            
            if input_buffer.button_pressed(Key::Space) || input_buffer.left_mouse_pressed() {
                {
                    let grid_lock = read_only_grid.lock().unwrap();
                    let cube_data = grid_lock.get_cube((sel.0 + view_x, sel.1 + view_y, sel.2 + view_z));
                    handle_selection(cube_data, &game_events, (sel.0 + view_x, sel.1 + view_y, sel.2 + view_z), &mut selection_state);
                }
            }

            if input_buffer.button_pressed(Key::X) {
                {
                    let mut grid_lock = read_only_grid.lock().unwrap();
                    if let None = grid_lock.get_cube((sel.0 + view_x, sel.1 + view_y, sel.2 + view_z)).agent {
                        grid_lock.delete_cube((sel.0 + view_x, sel.1 + view_y, sel.2 + view_z));
                    }
                }
            }

            if input_buffer.button_pressed(Key::P) {
                let world_pos = (sel.0 + view_x, sel.1 + view_y, sel.2 + view_z);
                if let Some(agent_id) = selection_state.selected_agent {
                    let _ = game_events.send(AgentAddTask {
                        task: AgentTask::Plough {
                            target: world_pos,
                        },
                        agent: agent_id as usize,
                    });
                }
            }
            if input_buffer.button_pressed(Key::K) {
                let world_pos = (sel.0 + view_x, sel.1 + view_y, sel.2 + view_z);
                if let Some(agent_id) = selection_state.selected_agent {
                    let _ = game_events.send(AgentAddTask {
                        task: AgentTask::Dig {
                            target: world_pos,
                        },
                        agent: agent_id as usize,
                    });
                }
            }
            if input_buffer.button_pressed(Key::L) {
                let world_pos = (sel.0 + view_x, sel.1 + view_y, sel.2 + 1 + view_z);
                if let Some(agent_id) = selection_state.selected_agent {
                    let _ = game_events.send(AgentAddTask {
                        task: AgentTask::Place {
                            target: world_pos,
                            cube: Cube {
                                cube_type: LANTERN_CUBE,
                                cube_x_face: None,
                                cube_y_face: None,
                                cube_z_face: None,
                                agent: None,
                                light_level: Light::min_level(),
                            }
                        },
                        agent: agent_id as usize,
                    });
                }
            }
            if input_buffer.button_pressed(Key::H) {
                view_x = GRID_WIDTH - VIEW_WIDTH;
                view_y = GRID_WIDTH - VIEW_WIDTH;
                view_z = GRID_HEIGHT - VIEW_HEIGHT;
            }
        }

        if let Some((sx, sy)) = window.get_mouse_pos(MouseMode::Clamp) {
           selected_cube = select_cube_mouse((sx as i32, sy as i32), &*grid.lock().unwrap(), (view_x, view_y, view_z));
        }

        let read_only_agents = Arc::clone(&agents);
        for z in 0..VIEW_HEIGHT {
            for y in 0..VIEW_WIDTH {
                for x in 0..VIEW_WIDTH {
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH) + (z + view_z) * GRID_WIDTH * GRID_WIDTH;

                    let (cube_screen_x, cube_screen_y) = get_screen_coord((x,y,z));

                    if selected_cube.is_some() {
                        if (x, y, z) == selected_cube.unwrap() {
                            draw_sprite((cube_screen_x, cube_screen_y), &select_cube, &mut buffer);
                            continue;
                        }
                    }


                    let cube_data = {
                        let grid = read_only_grid.lock().unwrap();
                        grid[cube_index]
                    };

                    if cube_data.cube_type == EMPTY_CUBE && !cube_data.agent.is_some() {
                        continue;
                    }

                    if let Some(agent) = cube_data.agent {
                        let agents_copy = read_only_agents.lock().unwrap();
                        let agent: &Agent = &agents_copy[agent as usize];
                        draw_sprite((cube_screen_x, cube_screen_y), &agent.animation.frames[agent.animation_state], &mut buffer);
                        continue;
                    }

                    let cube_light = if night_mode { cube_data.light_level } else { Light::max_level() };

                    {
                        let grid = read_only_grid.lock().unwrap();
                        if let Some(next_x) = grid.get_cube_next_x(cube_index) {
                            if next_x.is_transparent() || x == VIEW_WIDTH - 1 {
                                let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::RIGHT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::RIGHT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
                        }

                        if let Some(next_y) = grid.get_cube_next_y(cube_index) {
                            if next_y.is_transparent() || y == VIEW_WIDTH - 1 {
                                let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::LEFT, (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                            }
                        }
                        else {
                            let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::LEFT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                        }

                        if let Some(next_z) = grid.get_cube_above(cube_index) {
                            if next_z.is_transparent() {
                                let face = cube_data.cube_z_face.map_or(&sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::TOP,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                            }
                            else if z == VIEW_HEIGHT - 1 {
                                draw_face(Face::TOP,(cube_screen_x, cube_screen_y), &sprites[3], &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::TOP,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                        }
                    }
                }
            }
        }

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}


fn handle_selection(cube: &Cube, game_events: &Sender<AgentEvent>, selected_cube: (usize, usize, usize), selection_state: &mut InputState) {
    if let Some(agent) = cube.agent {
        selection_state.selected_agent = Some(agent);
        return;
    }
    match selection_state.selected_agent {
        Some(agent) => {
            game_events.send(AgentAddTask {
                task: AgentTask::FindPath {
                    destination: (selected_cube.0, selected_cube.1, selected_cube.2 + 1),
                },
                agent: agent as usize,
            });
            selection_state.selected_agent = None;
        },
        None => {}
    }

}

fn prepare_grid() -> Grid {
    let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT);


    let ground_level = GRID_HEIGHT - 20;
    let frequency_x: f32 = 0.2;
    let variance_x = 5.0;
    let frequency_y = 0.1;
    let variance_y = 6.0;
    let sea_level = ground_level - 10;


    for x in 0..GRID_WIDTH {
        for y in 0..GRID_WIDTH {
            for z in 0..GRID_HEIGHT {
                let cut_off = ground_level as f32
                    + (x as f32 * frequency_x).sin() * variance_x + (x as f32 * frequency_x * 4.0).sin() * variance_x / 8.0
                    + (y as f32 * frequency_y).sin() * variance_y+ (y as f32 * frequency_y * 4.0).sin() * variance_y / 8.0;
                let cut_off = cut_off as usize;
                let coord = (x,y,z);
                let index = grid.get_vector_pos(coord).unwrap();
                if z > cut_off {
                    if z > sea_level {
                        grid[index] = Cube::new(EMPTY_CUBE);
                    }
                    else {
                        grid[index] = Cube::new(6);
                    }
                }
                else if z == cut_off {
                    grid[index] = Cube::new(2);
                }
                else if z < cut_off && z > cut_off - 5 {
                    grid[index] = Cube::new(1);
                }
                else {
                    grid[index] = Cube::new(0);
                }
            }

        }
    }

    for i in 0..20 {
        let epicentre= (random_range(0..GRID_WIDTH), random_range(0..GRID_WIDTH), random_range(0..GRID_HEIGHT));
        let size = random_range(30..100);
        for i in 0..GRID_WIDTH {
            for j in 0..GRID_WIDTH {
                for k in 0..GRID_HEIGHT {
                    if (i as i32 - epicentre.0 as i32).pow(2) + (j as i32 - epicentre.1 as i32).pow(2) + (k as i32 - epicentre.2 as i32).pow(2) < size {
                        let index = grid.get_vector_pos((i,j,k)).unwrap();
                        grid[index] = Cube::new(EMPTY_CUBE);
                    }
                }
            }
        }
    }


    grid
}

fn find_ground_spawn_z(grid: &Grid, x: usize, y: usize) -> usize {
    for z in (0..GRID_HEIGHT).rev() {
        if grid.get_cube((x, y, z)).cube_type != EMPTY_CUBE {
            return (z + 1).min(GRID_HEIGHT - 1);
        }
    }
    0
}

fn get_screen_coord(world_space: (usize, usize, usize)) -> (usize, usize) {
    let x = world_space.0 as i32;
    let y = world_space.1 as i32;
    let z = world_space.2 as i32;

    let sx = (x - y) * (TILE_WIDTH / 2) as i32;
    let sy =  (x + y - 2 * z) * (TILE_HALF_WIDTH / 2) as i32;
    (
        (sx + (SCREEN_WIDTH / 2) as i32) as usize,
        ((sy + (SCREEN_HEIGHT / 2) as i32) - SCREEN_Y_OFFSET as i32) as usize,
    )
}


fn select_cube_mouse(screen_space: (i32, i32), grid: &Grid, view_point: (usize, usize, usize)) -> Option<(usize, usize, usize)> {
    let sx = screen_space.0 as f32 - SCREEN_WIDTH as f32 / 2.0 - TILE_HALF_WIDTH as f32;
    let sy = screen_space.1 as f32 - (SCREEN_HEIGHT as f32 / 2.0) + SCREEN_Y_OFFSET as f32 - TILE_HALF_WIDTH as f32;

    let a = sx / (TILE_WIDTH / 2) as f32;
    let b = sy / (TILE_HALF_WIDTH / 2) as f32;

    for z in (0..VIEW_HEIGHT as i32).rev() {

        let x = ((a + b + 2.0 * z as f32) / 2.0).round() as i32;
        let y = ((b - a + 2.0 * z as f32) / 2.0).round() as i32;

        if x < 0 || y < 0 {
            continue; //outside of the grid
        }

        let pos = (x as usize, y as usize, z as usize);

        let index = grid.get_vector_pos((x as usize + view_point.0, y as usize + view_point.1, z as usize + view_point.2));
        match index {
            Ok(_) => {
                let cube = grid.get_cube((x as usize + view_point.0, y as usize + view_point.1, z as usize + view_point.2));
                if cube.cube_type != EMPTY_CUBE || cube.agent.is_some() {
                    return Some(pos);
                }
            },
            Err(_) => {
                continue
            }
        }

    }
    None
}

