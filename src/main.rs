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
use minifb::{Key, MouseMode, Window, WindowOptions};
use minifb::Key::K;
use crate::agents::{find_path, Agent, AgentCoroutine, AgentEvent, AgentTask};
use crate::agents::AgentEvent::AgentAddTask;
use crate::animation::{Animation, AnimationPool};
use crate::grid::{Cube, Grid};
use crate::input::InputBuffer;
use crate::render::{draw_left_face, draw_right_face, draw_sprite, draw_top_face};

const SCREEN_WIDTH: usize = 2000;
const SCREEN_HEIGHT: usize = 1200;
const SCREEN_Y_OFFSET: usize = SCREEN_HEIGHT / 4;

const TILE_WIDTH: usize = 24;
const TILE_HALF_WIDTH: usize = TILE_WIDTH / 2;

const GRID_HEIGHT: usize = 120;
const GRID_WIDTH: usize = 120;

const VIEW_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;


const AGENT_TYPE_MASK: u64 = 0b10000000u8 as u64;

const CUBE_TYPE_MASK: u64 = 0b11111111u8 as u64;
const RIGHT_FACE_CUBE_MASK: u64 = CUBE_TYPE_MASK << 8;
const LEFT_FACE_CUBE_MASK: u64 = CUBE_TYPE_MASK << 16;
const TOP_FACE_CUBE_MASK: u64 = CUBE_TYPE_MASK << 24;

const EMPTY_CUBE: u8 = 255;


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
                    println!("locked grid");
                    match grid_lock {
                        Ok(mut grid) => {
                            println!("path len: {:?}", next);
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
                                println!("tryingf to move");
                                grid.move_cube(agent.position, next);
                                agent.position = next;

                            }
                            else {
                                println!("no path");
                                //recalculate path
                                //let path = find_path(agent.position, agent.destination.unwrap(), &grid);
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
                        agent.tasks.push(AgentTask::Move {
                            path: path,
                            destination: destination.clone(),
                        });
                    }

                    completed = true;
                }

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
    let man = Sprite::new("resources/24/man1.png");
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


    let sprites = vec![stone, mud, grass, blank, floor, man];

    let grid = Arc::new(Mutex::new(prepare_grid()));

    let plough_animation = Arc::new(Animation {
        frames: vec![man1, man2, man3, man4, man5, man6, man7, man8, man9],
        name: "ploughing".to_string(),
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
            ("running_ne".to_string(), run_animation_ne),
            ("running_nw".to_string(), run_animation_nw),
            ("running_se".to_string(), run_animation_se),
            ("running_sw".to_string(), run_animation_sw),
        ])
    });


    let mut man = Agent {
        animation: worker_animations.animations["ploughing"].clone(),
        position: (GRID_WIDTH-1, GRID_WIDTH-1, GRID_HEIGHT-4),
        animation_pool: Arc::clone(&worker_animations),
        name: "man".to_string(),
        animation_state: 0,
        destination: None,
        tasks: vec![],
        active_task: None,
        id: 0
    };

    grid.lock().unwrap().spawn_agent(&mut man);


    let (game_events, game_events_receiver) = mpsc::channel();
    let agents: Arc<Mutex<Vec<Agent>>> = Arc::new(Mutex::new(vec![man]));
    let agent_clone = Arc::clone(&agents);
    let grid_clone = Arc::clone(&grid);

    let mut gate_tick: u32 = 0;
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
                        if gate_tick < active_task.end_tick {
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
                        continue;
                    }
                    let mut next_task = agent.tasks.remove(0);
                    let task = AgentCoroutine {
                        task: next_task,
                        end_tick: gate_tick + 5,
                        completed: false,
                    };
                    println!("adding actibe task");
                    agent.active_task = Some(task);

                }
            }
            sleep(Duration::from_millis(300));
            gate_tick += 1;
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
    let mut selected_cube = (VIEW_WIDTH-1, VIEW_WIDTH-1, VIEW_HEIGHT-1);
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

        input_buffer.update_button_states(window.get_keys());

        if let Some(scroll_input) = scroll_input {
            if scroll_input > 0.0 && view_z < GRID_HEIGHT - VIEW_HEIGHT {
                view_z += 1;
            }
            if scroll_input < 0.0 && view_z > 0 {
                view_z -= 1;
            }
        }

        let mut buffer = buffer.clone();
        if input_buffer.button_pressed(Key::D) && view_x < GRID_WIDTH - VIEW_WIDTH {
            view_x += 1;
        }
        if  input_buffer.button_pressed(Key::A) && view_x > 0 {
            view_x -= 1;
        }
        if  input_buffer.button_pressed(Key::S) && view_y > 0 {
            view_y -= 1;
        }
        if  input_buffer.button_pressed(Key::W) && view_y < GRID_WIDTH - VIEW_WIDTH {
            view_y += 1;
        }

        if  input_buffer.button_pressed(Key::Right) && selected_cube.0 < VIEW_WIDTH {
            selected_cube = (selected_cube.0 + 1, selected_cube.1, selected_cube.2);
        }
        if  input_buffer.button_pressed(Key::Left) && selected_cube.0 > 0 {
            selected_cube = (selected_cube.0 - 1, selected_cube.1, selected_cube.2);
        }
        if  input_buffer.button_pressed(Key::Down) && selected_cube.1 < VIEW_WIDTH {
            selected_cube = (selected_cube.0, selected_cube.1 + 1, selected_cube.2);
        }
        if  input_buffer.button_pressed(Key::Up) && selected_cube.1 > 0 {
            selected_cube = (selected_cube.0, selected_cube.1 - 1, selected_cube.2);
        }

        if input_buffer.button_pressed(Key::Space) {
            {
                let grid_lock = read_only_grid.lock().unwrap();
                let cube_data = grid_lock.get_cube((selected_cube.0 + view_x, selected_cube.1 + view_y, selected_cube.2 + view_z));
                handle_selection(cube_data, &game_events, (selected_cube.0 + view_x, selected_cube.1 + view_y, selected_cube.2 + view_z), &mut selection_state);
            }

        }

        if let Some((sx, sy)) = window.get_mouse_pos(MouseMode::Clamp) {
           // select_cube((sx as i32, sy as i32), &mut grid, (view_x, view_y, view_z));
        }

        let read_only_agents = Arc::clone(&agents);
        for z in 0..VIEW_HEIGHT {
            for y in 0..VIEW_WIDTH {
                for x in 0..VIEW_WIDTH {
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH) + (z + view_z) * GRID_WIDTH * GRID_WIDTH;

                    let (cube_screen_x, cube_screen_y) = get_screen_coord((x,y,z));
                    if (x, y, z) == selected_cube {
                        draw_sprite((cube_screen_x, cube_screen_y), &select_cube, &mut buffer);
                        continue;
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
                                draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
                        }

                        if let Some(next_y) = grid.get_cube_next_y(cube_index) {
                            if next_y.is_transparent() || y == VIEW_WIDTH - 1 {
                                let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_left_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
                            }
                        }
                        else {
                            let face = cube_data.cube_y_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_left_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
                        }

                        if let Some(next_z) = grid.get_cube_above(cube_index) {
                            if next_z.is_transparent() || z == VIEW_HEIGHT - 1 {
                                let face = cube_data.cube_z_face.map_or(&sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                                draw_top_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
                            }
                        }
                        else {
                            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x as usize] });
                            draw_top_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
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


struct InputState {
    selected_agent: Option<u8>,
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
                    destination: selected_cube,
                },
                agent: agent as usize,
            });
            selection_state.selected_agent = None;
        },
        None => {}
    }

}


/**
fn draw_cube_face(face: Face, cube: &Cube, cube_index: usize, buffer: &mut [u8], grid: &Grid) {
    let blocking_cube = match face {
        Face::X => grid.get_cube_next_x(cube_index),
        Face::Y => grid.get_cube_next_y(cube_index),
        Face::Z => grid.get_cube_above(cube_index),
    };
    if let Some(blocker) = blocking_cube {
        !if blocker.is_transparent() {
            return;
        }
    }
    if grid[next_x].is_transparent() || x == VIEW_WIDTH - 1 {
            let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x] });
            draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
        }
    }
    else {
        let face = cube_data.cube_x_face.map_or( &sprites[cube_data.cube_type as usize], |x| { &sprites[x] });
        draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer, highlight_coolur)
    }
}
*/

enum Face {
    X,
    Y,
    Z
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

/**
fn select_cube(screen_space: (i32, i32), grid: &mut Grid, view_point: (usize, usize, usize)) -> (usize, usize, usize) {

    let sx = screen_space.0 - SCREEN_WIDTH as i32 / 2;
    let sy = screen_space.1 - (SCREEN_HEIGHT as i32 / 2) + SCREEN_Y_OFFSET as i32;

    println!("screen space {:?}, {:?}", sx, sy);

    let a = sx / (TILE_WIDTH/2) as i32;
    let b = sy / (TILE_HALF_WIDTH/2) as i32;
    let z = view_point.2 as i32 + (VIEW_HEIGHT - 1) as i32;

    let x = (a + b + 2 * z) / 2;
    let y = (b - a + 2 * z) / 2;

    if (x < 0 || y < 0) {
        return (0, 0, 0);
    }
    for i in 0..VIEW_HEIGHT as i32 {
        let px = x - i;
        let py = y - i;
        let pz = z - i;

        if px < 0 || py < 0 || pz < 0 {
            continue;
        }

        let pos = (px as usize, py as usize, pz as usize);
        let index = grid.get_vector_pos(pos);
        if (index < 0 || index >= cubes.len()) {
            return (0, 0, 0);
        }

        if cubes[grid.get_vector_pos(pos)] != EMPTY_CUBE {
            let index = grid.get_vector_pos(pos);
            if index < 0 || index >= cubes.len() {
                return (0, 0, 0);
            }
            cubes[grid.get_vector_pos(pos)] = 3;
            println!("hovered cube {:?}", pos);
            return (0, 0, 0);
        }
    }
    (0, 0, 0)
}
*/


struct Sprite {
    pub pixels: [u32; TILE_WIDTH * TILE_WIDTH],
}

impl Sprite {
    pub fn new(img: &str) -> Self {
        let img = open(Path::new(img)).expect(&format!("Error loading sprite {}", img)).into_rgba8();
        let mut pixels = [0u32; TILE_WIDTH * TILE_WIDTH];
        for (i, pixel) in img.pixels().enumerate() {
            let val = (pixel.0[3] as u32) << 24 | (pixel.0[0] as u32) << 16 | (pixel.0[1] as u32) << 8 | pixel.0[2] as u32;
            pixels[i] = val
        }

        Sprite {
            pixels
        }
    }
}