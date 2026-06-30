mod render;
mod agents;
mod grid;
mod animation;
mod input;
mod resources;
mod events;
mod world_generation;

use std::cmp::PartialEq;
use std::ops::Deref;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use noise::{NoiseFn};
use crate::agents::{find_path, Agent, AgentCoroutine, AgentEvent, AgentTask};
use crate::agents::AgentEvent::AgentAddTask;
use crate::agents::Direction::{Nx, Ny, Px, Py};
use crate::grid::{Cube, Grid, Light};
use crate::input::{InputBuffer, InputState};
use crate::render::{draw_left_face, draw_right_face, draw_sprite, draw_top_face, CubeFace, Sprite};
use crate::resources::{load_cube_sprites};
use crate::world_generation::{place_workers, prepare_grid};

const SCREEN_WIDTH: usize = 2000;
const SCREEN_HEIGHT: usize = 1200;
const SCREEN_Y_OFFSET: usize = SCREEN_HEIGHT / 4;
const SCREEN_X_OFFSET: usize = SCREEN_WIDTH / 4;

const TILE_WIDTH: usize = 24;
const TILE_HALF_WIDTH: usize = TILE_WIDTH / 2;

const GRID_HEIGHT: usize = 60;
const GRID_WIDTH: usize = 300;

const VIEW_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;

const EMPTY_CUBE: u8 = 255;
const STONE_CUBE: u8 = 0;
const WATER_CUBE: u8 = 6;
const LANTERN_CUBE: u8 = 7;

const SUN_LIGHT: (u8, u8, u8) = (205, 225, 235);


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
                                    agent.direction = Nx;
                                }
                                else if x_dir > 0 {
                                    agent.direction = Px;
                                }
                                else if y_dir < 0 {
                                    agent.direction = Ny;
                                }
                                else {
                                    agent.direction = Py;
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
                    }
                }

                completed = true;
            }
            AgentTask::Dig { target } => {
                if agent.position.2 > 0 {
                    if let Ok(mut grid) = grid.lock() {
                        grid.delete_cube(*target);
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

    let (grid_events_sender, grid_events_receiver) = mpsc::channel();

    let mut grid = prepare_grid(grid_events_sender);
    let agents = place_workers(&mut grid);

    let grid = Arc::new(Mutex::new(grid));
    let agents: Arc<Mutex<Vec<Agent>>> = Arc::new(Mutex::new(agents));

    let select_cube = Sprite::new("resources/24/select_cube.png");

    let (game_events, game_events_receiver) = mpsc::channel();
    let agent_clone = Arc::clone(&agents);
    let grid_clone = Arc::clone(&grid);
    let mut game_tick: u32 = 0;
    let agent_loop = thread::spawn(move || {
        loop {

            for event in grid_events_receiver.try_iter() {
                {
                    let mut grid_lock = grid_clone.lock().unwrap();
                    grid_lock.handle_grid_change_event(&event);
                }
            }
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
                            agent.change_animation("running");
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

    let mut view_x = GRID_WIDTH - VIEW_WIDTH;
    let mut view_y = GRID_WIDTH - VIEW_WIDTH;
    let mut view_z = GRID_HEIGHT - VIEW_HEIGHT;

    let mut selected_cube: Option<WorldPos> = None;
    let mut night_mode = false;

    let mut compass = Compass::North;
    let mut input_buffer = InputBuffer::new();
    let mut selection_state = InputState {
        selected_agent: None
    };

    let read_only_grid = Arc::clone(&grid);

    let read_only_agents = Arc::clone(&agents);
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

        if let Some((sx, sy)) = window.get_mouse_pos(MouseMode::Clamp) {
            selected_cube = select_cube_mouse(&compass, ScreenPos::new(sx as u32, sy as u32), &*grid.lock().unwrap(), (view_x, view_y, view_z));
        }

        if let Some(ref sel) = selected_cube {
            if input_buffer.button_pressed(Key::Space) || input_buffer.left_mouse_pressed() {
                {
                    let grid_lock = read_only_grid.lock().unwrap();
                    let cube_data = grid_lock.get_cube((sel.x + view_x, sel.y + view_y, sel.z + view_z));
                    handle_selection(cube_data, &game_events, (sel.x + view_x, sel.y + view_y, sel.z + view_z), &mut selection_state);
                }
            }

            if input_buffer.button_pressed(Key::X) {
                {
                    let mut grid_lock = read_only_grid.lock().unwrap();
                    if let None = grid_lock.get_cube((sel.x + view_x, sel.y + view_y, sel.z + view_z)).agent {
                        grid_lock.delete_cube((sel.x + view_x, sel.y + view_y, sel.z + view_z));
                    }
                }
            }

            if input_buffer.right_mouse_pressed() {
                if selection_state.selected_agent.is_some() {
                    selection_state.selected_agent = None;
                }
                else {
                    let above = (sel.x + view_x, sel.y + view_y, sel.z + view_z + 1);
                    if above.2 < GRID_HEIGHT {
                        let mut grid_lock = read_only_grid.lock().unwrap();
                        if !grid_lock.is_occupied(above) {
                            grid_lock.place_cube(above, Cube::new(STONE_CUBE));
                        }
                    }
                }

            }

            if input_buffer.button_pressed(Key::R) {
                match &compass {
                    Compass::North => compass = Compass::East,
                    Compass::East => compass = Compass::South,
                    Compass::South => compass = Compass::West,
                    Compass::West => compass = Compass::North
                }
            }

            if input_buffer.button_pressed(Key::P) {
                let world_pos = (sel.x + view_x, sel.y + view_y, sel.z + view_z);
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
                let world_pos = (sel.x + view_x, sel.y + view_y, sel.z + view_z);
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
                let world_pos = (sel.x + view_x, sel.y + view_y, sel.z + 1 + view_z);
                if let Some(agent_id) = selection_state.selected_agent {
                    let _ = game_events.send(AgentAddTask {
                        task: AgentTask::Place {
                            target: world_pos,
                            cube: Cube::new(LANTERN_CUBE)
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

        //render
        let mut buffer = vec![0xFFFFFF; SCREEN_WIDTH * SCREEN_HEIGHT];
        let flip_faces = compass != Compass::North && compass != Compass::South;


        for z in 0..VIEW_HEIGHT {
            for mut y in 0..VIEW_WIDTH {
                if compass == Compass::East || compass == Compass::South {
                    y = (VIEW_WIDTH - 1) - y;
                }
                for mut x in 0..VIEW_WIDTH {
                    if  compass == Compass::South || compass == Compass::West {
                        x = (VIEW_WIDTH - 1) - x;
                    }
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH) + (z + view_z) * GRID_WIDTH * GRID_WIDTH;

                    let (cube_screen_x, cube_screen_y) = get_screen_coord(&compass, (x,y,z));

                    if selected_cube.is_some() {
                        if WorldPos::new(x,y,z) == selected_cube.clone().unwrap() {
                            draw_sprite((cube_screen_x, cube_screen_y), &select_cube, &mut buffer, (0,0,0));
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
                        let highlight_colour = selection_state.selected_agent
                            .filter(|selected_agent| *selected_agent == agent.id as u8)
                            .map_or((0,0,0), |_| (125, 0, 0));
                        draw_sprite((cube_screen_x, cube_screen_y), &agent.get_current_animation_frame(&compass), &mut buffer, highlight_colour);
                        continue;
                    }

                    let cube_light = if night_mode { cube_data.light_level } else { Light::max_level() };


                    let (visible_face_left, visible_face_right, terminal_x_layer, terminal_y_layer) = match &compass {
                        Compass::North => (CubeFace::pY, CubeFace::pX, VIEW_WIDTH - 1, VIEW_WIDTH - 1),
                        Compass::East => (CubeFace::pX, CubeFace::nY, VIEW_WIDTH - 1, 0),
                        Compass::South =>  (CubeFace::nY, CubeFace::nX, 0, 0),
                        Compass::West => (CubeFace::nX, CubeFace::pY, 0, VIEW_WIDTH - 1)
                    };

                    let (blocking_left, blocking_right, blocking_top) = {
                        let grid = read_only_grid.lock().unwrap();
                        let left = grid.get_blocking_cube(&visible_face_left, cube_index);
                        let right = grid.get_blocking_cube(&visible_face_right, cube_index);
                        let top = grid.get_blocking_cube(&CubeFace::Z, cube_index);
                        (left, right, top)
                    };

                    if let Some(next_y) = blocking_left {
                        if next_y.is_transparent() {
                            let face = cube_data.cube_y_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                            draw_left_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
                        }
                    }
                    else {
                        let face = cube_data.cube_y_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                        draw_left_face((cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
                    }


                    if let Some(next_x) = blocking_right {
                        if next_x.is_transparent() {
                            let face = cube_data.cube_x_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                            draw_right_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                        }
                    }
                    else {
                        let face = cube_data.cube_x_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                        draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                    }

                    if let Some(next_z) = blocking_top {
                        if next_z.is_transparent() {
                            let face = cube_data.cube_z_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                            draw_top_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                        }
                        else if z == VIEW_HEIGHT - 1 {
                            draw_top_face((cube_screen_x, cube_screen_y), &sprites[3], &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                        }
                    }
                    else {
                        let face = cube_data.cube_z_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                        draw_top_face((cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.z_level, cube_light.z_level, cube_light.z_level))
                    }

                    if x == terminal_x_layer {
                        let face = cube_data.cube_x_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                        if flip_faces {
                            draw_left_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
                        }
                        else {
                            draw_right_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                        }
                    }

                    if y == terminal_y_layer {
                        let face = cube_data.cube_y_face.map_or( find_sprite(&cube_data, &sprites), |x| { &sprites[x as usize] });
                        if flip_faces {
                            draw_right_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.y_level, cube_light.y_level, cube_light.y_level))
                        }
                        else {
                            draw_left_face( (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_light.x_level, cube_light.x_level, cube_light.x_level))
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

fn find_sprite<'a>(cube_data: &Cube, sprites: &'a Vec<Sprite>) -> &'a Sprite {
    &sprites[cube_data.cube_type as usize]
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



#[derive(PartialEq)]
enum Compass {
    North,
    South,
    East,
    West,
}

//this is the transposed version M^T * worldPos -> Camera Space
//then project onto camera xy plane
fn get_screen_coord(compass: &Compass, world_space: (usize, usize, usize)) -> (usize, usize) {
    let world_space = (world_space.0 as i32, world_space.1 as i32, world_space.2 as i32);

    let (camera_x_dir, camera_y_dir, x_offset, y_offset) = match compass {
        Compass::North => ((1i32, -1i32, 0i32), (1i32,  1i32, -2i32), 0, -1i32 * SCREEN_Y_OFFSET as i32),
        Compass::East => ((-1i32, -1i32, 0i32), (1i32,  -1i32, -2i32), SCREEN_X_OFFSET as i32, 0 as i32),
        Compass::South => ((-1i32, 1i32, 0i32), (-1i32,  -1i32, -2i32), 0, (SCREEN_Y_OFFSET as f32 * 1.5) as i32), //TODO this will cause rounding errors going back to mouse
        Compass::West => ((1i32, 1i32, 0i32), (-1i32,  1i32, -2i32), -1i32 * SCREEN_X_OFFSET as i32, 0 as i32),
    };
    let sx = dot(camera_x_dir, world_space);
    let sy = dot(camera_y_dir, world_space);
    let sx = sx * (TILE_WIDTH / 2) as i32 + ((SCREEN_WIDTH / 2) as i32);
    let sy = sy  * (TILE_HALF_WIDTH / 2) as i32 + ((SCREEN_HEIGHT / 2) as i32);

    (
        (sx + x_offset) as usize,
        (sy + y_offset) as usize
    )
}

fn dot(p0: (i32, i32, i32), p1: (i32, i32, i32)) -> i32 {
    p0.0 * p1.0 + p0.1 * p1.1 + p0.2 * p1.2
}


//even though this returns a 'world pos' it is relative to the view window
fn select_cube_mouse(compass: &Compass, screen_space: ScreenPos, grid: &Grid, view_point: (usize, usize, usize)) -> Option<WorldPos> {
    let (sx, sy) = match compass {
        Compass::North => (
            screen_space.x as f32 - SCREEN_WIDTH as f32 / 2.0,
            screen_space.y as f32 - (SCREEN_HEIGHT as f32 / 2.0) + SCREEN_Y_OFFSET as f32
        ),
        Compass::East => (
            screen_space.x as f32 - (SCREEN_WIDTH as f32 / 2.0) - SCREEN_X_OFFSET as f32,
            screen_space.y as f32 - (SCREEN_HEIGHT as f32 / 2.0)
        ),
        Compass::South => (
            screen_space.x as f32 - (SCREEN_WIDTH as f32 / 2.0),
            screen_space.y as f32 - (SCREEN_HEIGHT as f32 / 2.0) - (SCREEN_Y_OFFSET as f32 * 1.5)
        ),
        Compass::West => (
            screen_space.x as f32 - (SCREEN_WIDTH as f32 / 2.0) + SCREEN_X_OFFSET as f32,
            screen_space.y as f32 - (SCREEN_HEIGHT as f32 / 2.0)
        ),
    };

    //not in inverse but shift mousePos over by half tile so that boundry isnt in centre of sprite
    let sx = sx - TILE_HALF_WIDTH as f32;
    let sy = sy - TILE_HALF_WIDTH as f32;

    let sx = sx / (TILE_WIDTH / 2) as f32;
    let sy = sy / (TILE_HALF_WIDTH / 2) as f32;

    for z in (0..VIEW_HEIGHT as i32).rev() {

        let (x, y) = match compass {
            Compass::North => (
                ((sx + sy + 2.0 * z as f32) / 2.0).round() as i32,
                ((sy - sx + 2.0 * z as f32) / 2.0).round() as i32
            ),
            Compass::East => (
                ((sy - sx + 2.0 * z as f32) / 2.0).round() as i32,
                ((-sy - sx - 2.0 * z as f32) / 2.0).round() as i32
            ),
            Compass::South => (
                ((-sx - sy - 2.0 * z as f32) / 2.0).round() as i32,
                ((sx - sy - 2.0 * z as f32) / 2.0).round() as i32
            ),
            Compass::West => (
                ((sx - sy - 2.0 * z as f32) / 2.0).round() as i32,
                ((sy + sx + 2.0 * z as f32) / 2.0).round() as i32
            )

        };

        if x < 0 || y < 0 {
            continue; //outside of the grid
        }

        let pos = (x as usize, y as usize, z as usize);

        let index = grid.get_vector_pos((x as usize + view_point.0, y as usize + view_point.1, z as usize + view_point.2));
        match index {
            Ok(_) => {
                let cube = grid.get_cube((x as usize + view_point.0, y as usize + view_point.1, z as usize + view_point.2));
                if cube.cube_type != EMPTY_CUBE || cube.agent.is_some() {
                    println!("{:?}", pos);
                    return Some(WorldPos::new(x as usize, y as usize, z as usize));
                }
            },
            Err(_) => {
                continue
            }
        }

    }
    None
}

#[derive(Clone)]
struct ScreenPos {
    pub x: u32,
    pub y: u32,
}

impl ScreenPos {
    pub fn new(x: u32, y: u32) -> ScreenPos {
        ScreenPos { x, y }
    }
}


#[derive(Clone)]
struct WorldPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl WorldPos {
    fn new(x: usize, y: usize, z: usize) -> WorldPos {
        WorldPos { x, y, z }
    }
}

impl PartialEq for WorldPos {
    fn eq(&self, other: &WorldPos) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}
