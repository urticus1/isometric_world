mod render;
mod agents;
mod grid;
mod animation;
mod input;

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
use crate::agents::{find_path, Agent, AgentCoroutine, AgentEvent, AgentTask};
use crate::agents::AgentEvent::AgentAddTask;
use crate::animation::{Animation, AnimationPool};
use crate::grid::{find_horizontal_neighbours, get_manhattan_distance, is_horizontal_neighbour, Cube, Grid};
use crate::input::{InputBuffer, InputState};
use crate::render::{draw_face, draw_sprite, light_flood_fill, Face, Sprite};

const SCREEN_WIDTH: usize = 2000;
const SCREEN_HEIGHT: usize = 1200;
const SCREEN_Y_OFFSET: usize = SCREEN_HEIGHT / 4;

const TILE_WIDTH: usize = 24;
const TILE_HALF_WIDTH: usize = TILE_WIDTH / 2;

const GRID_HEIGHT: usize = 120;
const GRID_WIDTH: usize = 120;

const VIEW_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;

const EMPTY_CUBE: u8 = 255;

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
                        let index = grid.get_vector_pos(*target);
                        grid[index].cube_type = 4;
                    }
                }
                completed = true;
            }
            AgentTask::Dig { target } => {
                if agent.position.2 > 0 {
                    if let Ok(mut grid) = grid.lock() {
                        let index = grid.get_vector_pos(*target);
                        grid[index].cube_type = EMPTY_CUBE;
                    }
                }
                completed = true;
            }
            AgentTask::Place { target, cube } => {

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
    let stone = Sprite::new("resources/24/stone.png");
    let mud = Sprite::new("resources/24/mud.png");
    let grass = Sprite::new("resources/24/grass.png");
    let blank = Sprite::new("resources/24/blank.png");
    let floor = Sprite::new("resources/24/floor.png");
    let man = Sprite::new("resources/24/man/man_idle.png");
    let select_cube = Sprite::new("resources/24/select_cube.png");


    let man1 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing1.png");
    let man2 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing2.png");
    let man3 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing3.png");
    let man4 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing4.png");
    let man5 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing5.png");
    let man6 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing6.png");
    let man7 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing7.png");
    let man8 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing8.png");
    let man9 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing9.png");

    let man_mining1 = Sprite::new("resources/24/man/animations/mining/man_mining1.png");
    let man_mining2 = Sprite::new("resources/24/man/animations/mining/man_mining2.png");
    let man_mining3 = Sprite::new("resources/24/man/animations/mining/man_mining3.png");
    let man_mining4 = Sprite::new("resources/24/man/animations/mining/man_mining4.png");
    let man_mining5 = Sprite::new("resources/24/man/animations/mining/man_mining5.png");
    let man_mining6 = Sprite::new("resources/24/man/animations/mining/man_mining6.png");
    let man_mining7 = Sprite::new("resources/24/man/animations/mining/man_mining7.png");
    let man_mining8 = Sprite::new("resources/24/man/animations/mining/man_mining8.png");
    let man_mining9 = Sprite::new("resources/24/man/animations/mining/man_mining9.png");
    let man_mining10 = Sprite::new("resources/24/man/animations/mining/man_mining10.png");
    let man_mining11 = Sprite::new("resources/24/man/animations/mining/man_mining11.png");

    let man_r_ne1 = Sprite::new("resources/24/man/animations/running/man_running_ne1.png");
    let man_r_ne2 = Sprite::new("resources/24/man/animations/running/man_running_ne2.png");
    let man_r_ne3 = Sprite::new("resources/24/man/animations/running/man_running_ne3.png");
    let man_r_ne4 = Sprite::new("resources/24/man/animations/running/man_running_ne4.png");
    let man_r_ne5 = Sprite::new("resources/24/man/animations/running/man_running_ne5.png");
    let man_r_ne6 = Sprite::new("resources/24/man/animations/running/man_running_ne6.png");

    let man_r_nw1 = Sprite::new("resources/24/man/animations/running/man_running_nw1.png");
    let man_r_nw2 = Sprite::new("resources/24/man/animations/running/man_running_nw2.png");
    let man_r_nw3 = Sprite::new("resources/24/man/animations/running/man_running_nw3.png");
    let man_r_nw4 = Sprite::new("resources/24/man/animations/running/man_running_nw4.png");
    let man_r_nw5 = Sprite::new("resources/24/man/animations/running/man_running_nw5.png");
    let man_r_nw6 = Sprite::new("resources/24/man/animations/running/man_running_nw6.png");

    let man_r_sw1 = Sprite::new("resources/24/man/animations/running/man_running_sw1.png");
    let man_r_sw2 = Sprite::new("resources/24/man/animations/running/man_running_sw2.png");
    let man_r_sw3 = Sprite::new("resources/24/man/animations/running/man_running_sw3.png");
    let man_r_sw4 = Sprite::new("resources/24/man/animations/running/man_running_sw4.png");
    let man_r_sw5 = Sprite::new("resources/24/man/animations/running/man_running_sw5.png");
    let man_r_sw6 = Sprite::new("resources/24/man/animations/running/man_running_sw6.png");
    
    let man_r_se1 = Sprite::new("resources/24/man/animations/running/man_running_se1.png");
    let man_r_se2 = Sprite::new("resources/24/man/animations/running/man_running_se2.png");
    let man_r_se3 = Sprite::new("resources/24/man/animations/running/man_running_se3.png");
    let man_r_se4 = Sprite::new("resources/24/man/animations/running/man_running_se4.png");
    let man_r_se5 = Sprite::new("resources/24/man/animations/running/man_running_se5.png");
    let man_r_se6 = Sprite::new("resources/24/man/animations/running/man_running_se6.png");

    let man_idle = Sprite::new("resources/24/man/man_idle.png");

    let sprites = vec![stone, mud, grass, blank, floor, man];

    let grid = Arc::new(Mutex::new(prepare_grid()));

    let plough_animation = Arc::new(Animation {
        frames: vec![man1, man2, man3, man4, man5, man6, man7, man8, man9],
        name: "ploughing".to_string(),
    });

    let mining_animation = Arc::new(Animation {
        frames: vec![man_mining1, man_mining2, man_mining3, man_mining4, man_mining5, man_mining6, man_mining7, man_mining8, man_mining9, man_mining10, man_mining11],
        name: "mining".to_string(),
    });

    let idle_animation = Arc::new(Animation {
        frames: vec![man_idle],
        name: "idle".to_string(),
    });

    let run_animation_ne = Arc::new(Animation {
        frames: vec![man_r_ne1, man_r_ne2, man_r_ne3, man_r_ne4, man_r_ne5, man_r_ne6],
        name: "running_ne".to_string(),
    });
    let run_animation_nw = Arc::new(Animation {
        frames: vec![man_r_nw1, man_r_nw2, man_r_nw3, man_r_nw4, man_r_nw5, man_r_nw6],
        name: "running_nw".to_string(),
    });
    let run_animation_sw = Arc::new(Animation {
        frames: vec![man_r_sw1, man_r_sw2, man_r_sw3, man_r_sw4, man_r_sw5, man_r_sw6],
        name: "running_sw".to_string(),
    });
    let run_animation_se = Arc::new(Animation {
        frames: vec![man_r_se1, man_r_se2, man_r_se3, man_r_se4, man_r_se5, man_r_se6],
        name: "running_se".to_string(),
    });

    let worker_animations = Arc::new(AnimationPool {
        animations: HashMap::from([
            ("ploughing".to_string(), plough_animation),
            ("mining".to_string(), mining_animation),
            ("running_ne".to_string(), run_animation_ne),
            ("running_nw".to_string(), run_animation_nw),
            ("running_se".to_string(), run_animation_se),
            ("running_sw".to_string(), run_animation_sw),
            ("idle".to_string(), idle_animation),
        ])
    });


    let mut man = Agent {
        animation: worker_animations.animations["idle"].clone(),
        position: (GRID_WIDTH-1, GRID_WIDTH-1, GRID_HEIGHT-4),
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
        position: (GRID_WIDTH-5, GRID_WIDTH-5, GRID_HEIGHT-4),
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
                                        break;
                                    }
                                }
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
                    agent.active_task = Some(task);

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
    let mut highlight_coolur = 0;
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
            if input_buffer.button_pressed(Key::L) {
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

                    {
                        let grid = read_only_grid.lock().unwrap();
                        if let Some(next_x) = grid.get_cube_next_x(cube_index) {
                            if next_x.is_transparent() || x == VIEW_WIDTH - 1 {
                                let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::RIGHT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::RIGHT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
                        }

                        if let Some(next_y) = grid.get_cube_next_y(cube_index) {
                            if next_y.is_transparent() || y == VIEW_WIDTH - 1 {
                                let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::LEFT, (cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
                            }
                        }
                        else {
                            let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::LEFT,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
                        }

                        if let Some(next_z) = grid.get_cube_above(cube_index) {
                            if next_z.is_transparent() || z == VIEW_HEIGHT - 1 {
                                let face = cube_data.cube_z_face.map_or(&sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_face(Face::TOP,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_face(Face::TOP,(cube_screen_x, cube_screen_y), face, &mut buffer, (cube_data.light_level.level, cube_data.light_level.level, cube_data.light_level.level))
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

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_WIDTH {
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 1));
            grid[index] = Cube::new(EMPTY_CUBE);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 2));
            grid[index] = Cube::new(EMPTY_CUBE);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 3));
            grid[index] = Cube::new(EMPTY_CUBE);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 4));
            grid[index] = Cube::new(EMPTY_CUBE);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 5));
            grid[index] = Cube::new(2);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 6));
            grid[index] = Cube::new(2);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 7));
            grid[index] = Cube::new(1);
            let index = grid.get_vector_pos((i,j,GRID_HEIGHT - 8));
            grid[index] = Cube::new(1);
        }
    }

    let epicentre= (GRID_WIDTH / 2, GRID_WIDTH / 2, GRID_HEIGHT -1);

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_WIDTH {
            for k in 0..GRID_HEIGHT {
                if (i as i32 - epicentre.0 as i32).pow(2) + (j as i32 - epicentre.1 as i32).pow(2) + (k as i32 - epicentre.2 as i32).pow(2) < 60 {
                    let index = grid.get_vector_pos((i,j,k));
                    grid[index] = Cube::new(EMPTY_CUBE);
                }
            }
        }
    }
    light_flood_fill((GRID_WIDTH-1, GRID_WIDTH-1, GRID_HEIGHT - 4), &mut grid);
    grid
}

fn get_screen_coord(world_space: (usize, usize, usize)) -> (usize, usize) {
    let x = world_space.0 as i32;
    let y = world_space.1 as i32;
    let z = world_space.2 as i32;

    let sx = (x - y) * (TILE_WIDTH / 2) as i32;
    let sy =  (x + y - 2 * z) * (TILE_HALF_WIDTH / 2) as i32;
    (
        (sx + (SCREEN_WIDTH / 2) as i32) as usize,
        (sy + (SCREEN_HEIGHT / 2) as i32) as usize - SCREEN_Y_OFFSET,
    )
}


fn select_cube_mouse(screen_space: (i32, i32), grid: &Grid, view_point: (usize, usize, usize)) -> Option<(usize, usize, usize)> {
    let sx = screen_space.0 as f32 - SCREEN_WIDTH as f32 / 2.0 - TILE_HALF_WIDTH as f32;
    let sy = screen_space.1 as f32 - (SCREEN_HEIGHT as f32 / 2.0) + SCREEN_Y_OFFSET as f32 - TILE_HALF_WIDTH as f32;

    let a = sx / (TILE_WIDTH / 2) as f32;
    let b = sy / (TILE_HALF_WIDTH / 2) as f32;

    for z in (0..VIEW_HEIGHT as i32).rev() {
        let pz = z;

        let x = ((a + b + 2.0 * z as f32) / 2.0).round() as i32;
        let y = ((b - a + 2.0 * z as f32) / 2.0).round() as i32;

        if x < 0 || y < 0 || pz < 0 {
            continue;
        }

        let pos = (x as usize, y as usize, pz as usize);

        let index = grid.get_vector_pos((x as usize + view_point.0, y as usize + view_point.1, pz as usize + view_point.2));
        if index >= GRID_WIDTH * GRID_WIDTH * GRID_HEIGHT {
            return None;
        }
        let cube = grid.get_cube((x as usize + view_point.0, y as usize + view_point.1, pz as usize + view_point.2));
        if cube.cube_type != EMPTY_CUBE || cube.agent.is_some() {
            return Some(pos);
        }
    }
    None
}

