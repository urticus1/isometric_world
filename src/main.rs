mod render;

use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use image::{open, Frame};
use minifb::{Key, MouseMode, Window, WindowOptions};
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

const EMPTY_CUBE: u64 = 255;

fn main() {
    let stone = Sprite::new("resources/24/stone.png");
    let mud = Sprite::new("resources/24/mud.png");
    let grass = Sprite::new("resources/24/grass.png");
    let blank = Sprite::new("resources/24/blank.png");
    let floor = Sprite::new("resources/24/floor.png");
    let man = Sprite::new("resources/24/man1.png");


    let man1 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing1.png");
    let man2 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing2.png");
    let man3 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing3.png");
    let man4 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing4.png");
    let man5 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing5.png");
    let man6 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing6.png");
    let man7 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing7.png");
    let man8 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing8.png");
    let man9 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing9.png");


    let sprites = vec![stone, mud, grass, blank, floor, man];

    let mut cubes = vec![0u64; GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH];

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_WIDTH {

            cubes[get_vector_pos((i,j , GRID_HEIGHT - 1))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 2))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 3))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 4))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 5))] = 2;// + (4 << 24);
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 6))] = 2;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 7))] = 1;
            cubes[get_vector_pos((i,j,GRID_HEIGHT - 8))] = 1;
        }
    }

    cubes[get_vector_pos((GRID_WIDTH-1, GRID_HEIGHT - 1,GRID_WIDTH-1))] = AGENT_TYPE_MASK;

    let plough_animation = Animation {
        frames: vec![man1, man2, man3, man4, man5, man6, man7, man8, man9],
        name: "ploughing".to_string(),
    };

    let mut man = Agent {
        animation: plough_animation,
        position: (0,0,0),
        name: "man".to_string(),
        animation_state: 0
    };

    let epicentre= (GRID_WIDTH / 2, GRID_HEIGHT -1, GRID_WIDTH / 2);

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            for k in 0..GRID_WIDTH {
                if (i as i32 - epicentre.0 as i32).pow(2) + (j as i32 - epicentre.1 as i32).pow(2) + (k as i32 - epicentre.2 as i32).pow(2) < 60 {
                    cubes[get_vector_pos((i,j,k))] = EMPTY_CUBE;
                }
            }
        }
    }

    let agents: Arc<Mutex<Vec<Agent>>> = Arc::new(Mutex::new(vec![man]));
    let agent_clone = Arc::clone(&agents);
    let agent_loop = thread::spawn(move || {
        loop {
            {
                let mut mut_agents =  agent_clone.lock().unwrap();
                for mut agent in mut_agents.iter_mut() {
                    if agent.animation_state == agent.animation.frames.len() - 1 {
                        agent.animation_state = 0
                    }
                    else {
                        agent.animation_state = agent.animation_state + 1
                    }
                }
            }
            sleep(Duration::from_millis(300));
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
    while window.is_open() && !window.is_key_down(Key::Escape) {

        let mut buffer = buffer.clone();
        if window.get_keys().contains(&Key::Right) && view_x < GRID_WIDTH - VIEW_WIDTH {
            view_x += 1;
        }
        if window.get_keys().contains(&Key::Left) && view_x > 0 {
            view_x -= 1;
        }
        if window.get_keys().contains(&Key::W) && view_z < GRID_HEIGHT - VIEW_HEIGHT {
            view_z += 1;
        }
        if window.get_keys().contains(&Key::S) && view_z > 0 {
            view_z -= 1;
        }
        if window.get_keys().contains(&Key::Down) && view_y > 0 {
            view_y -= 1;
        }
        if window.get_keys().contains(&Key::Up) && view_y < GRID_WIDTH - VIEW_WIDTH {
            view_y += 1;
        }

        if let Some((sx, sy)) = window.get_mouse_pos(MouseMode::Clamp) {
            //select_cube((sx as i32, sy as i32), &mut cubes, (view_x, view_y, view_z));
        }

        let read_only_agents = Arc::clone(&agents);
        for z in 0..VIEW_HEIGHT {
            for y in 0..VIEW_WIDTH {
                for x in 0..VIEW_WIDTH {
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH) + (z + view_z) * GRID_WIDTH * GRID_WIDTH;

                    let cube_data = cubes[cube_index];
                    let cube_type = cube_data & CUBE_TYPE_MASK;
                    if (cube_type == EMPTY_CUBE) {
                        continue;
                    }
                    if (cubes[get_vector_pos((x + 1, y + 1, z + 1))] & CUBE_TYPE_MASK) == EMPTY_CUBE {
                        continue;
                    }
                    let (cube_screen_x, cube_screen_y) = get_screen_coord((x,y,z));
                    if cube_type & AGENT_TYPE_MASK != 0 {
                        let index = (cube_type &! AGENT_TYPE_MASK) as usize;
                        {
                            let agents_copy = read_only_agents.lock().unwrap();
                            let agent: &Agent = &agents_copy[index];
                            draw_sprite((cube_screen_x, cube_screen_y), &agent.animation.frames[agent.animation_state], &mut buffer)
                        }
                    }
                    else {
                        if let Some(next_x) = get_cube_next_x(cube_index) {

                            if cubes[next_x] & CUBE_TYPE_MASK == EMPTY_CUBE || x == VIEW_WIDTH - 1 {
                                let mut face = &sprites[cube_type as usize];
                                if cube_data & RIGHT_FACE_CUBE_MASK != 0 {
                                    face = &sprites[((cube_data & RIGHT_FACE_CUBE_MASK) >> 8) as usize]
                                }
                                draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer)
                            }
                        }
                        else {
                            let mut face = &sprites[cube_type as usize];
                            if cube_data & RIGHT_FACE_CUBE_MASK != 0 {
                                face = &sprites[((cube_data & RIGHT_FACE_CUBE_MASK) >> 8) as usize]
                            }
                            draw_right_face((cube_screen_x, cube_screen_y), face, &mut buffer)
                        }


                        if let Some(next_z) = get_cube_next_y(cube_index) {
                            if cubes[next_z] & CUBE_TYPE_MASK == EMPTY_CUBE || y == VIEW_WIDTH - 1 {
                                let mut face = &sprites[cube_type as usize];
                                if (cube_data & LEFT_FACE_CUBE_MASK) != 0 {
                                    face = &sprites[((cube_data & LEFT_FACE_CUBE_MASK) >> 16) as usize];
                                }
                                draw_left_face((cube_screen_x, cube_screen_y), face, &mut buffer)
                            }
                        }
                        else {
                            let mut face = &sprites[cube_type as usize];
                            if (cube_data & LEFT_FACE_CUBE_MASK) != 0 {
                                face = &sprites[((cube_data & LEFT_FACE_CUBE_MASK) >> 16) as usize];
                            }
                            draw_left_face((cube_screen_x, cube_screen_y), face, &mut buffer)
                        }

                        if let Some(next_y) = get_cube_above(cube_index) {
                            if cubes[next_y] & CUBE_TYPE_MASK == EMPTY_CUBE || z == VIEW_HEIGHT - 1 {
                                let mut face = &sprites[cube_type as usize];
                                if cube_data & TOP_FACE_CUBE_MASK != 0 {
                                    face = &sprites[((cube_data & TOP_FACE_CUBE_MASK) >> 24) as usize]
                                }
                                draw_top_face((cube_screen_x, cube_screen_y), face, &mut buffer)
                            }
                        }
                        else {
                            let mut face = &sprites[cube_type as usize];
                            if cube_data & TOP_FACE_CUBE_MASK != 0 {
                                face = &sprites[((cube_data & TOP_FACE_CUBE_MASK) >> 24) as usize]
                            }
                            draw_top_face((cube_screen_x, cube_screen_y), face, &mut buffer)
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


fn get_cube_next_x(i: usize) -> Option<usize> {
    if i % GRID_WIDTH == GRID_WIDTH -1 {
        return None
    }
    Some(i + 1)
}

fn get_cube_next_y(i: usize) -> Option<usize> {
    if i % (GRID_WIDTH * GRID_WIDTH) + GRID_WIDTH >= GRID_WIDTH * GRID_WIDTH {
        return None
    }
    Some(i + GRID_WIDTH)
}

fn get_cube_above(i: usize) -> Option<usize> {
    let val = i + GRID_WIDTH * GRID_WIDTH;
    if val >= GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH {
        return None;
    }
    Some(i + GRID_WIDTH * GRID_WIDTH)
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

fn select_cube(screen_space: (i32, i32), cubes: &mut Vec<u64>, view_point: (usize, usize, usize)) -> (usize, usize, usize) {

    let sx = screen_space.0 - SCREEN_WIDTH as i32 / 2;
    let sy = screen_space.1 - (SCREEN_HEIGHT as i32 / 2) + SCREEN_Y_OFFSET as i32;

    let a = sx / TILE_WIDTH as i32;
    let b = sy / TILE_HALF_WIDTH as i32;
    let z = view_point.2 as i32 + VIEW_HEIGHT as i32;

    let x = (a + b + 2 * z) / 2;
    let y = (b - a + 2 * z) / 2;

    if (x < 0 || y < 0) {
        return (0, 0, 0);
    }
    let start_cube = (x as usize, y as usize, z as usize);
    for i in 0..5 {
        if (i > start_cube.0 || i > start_cube.1 || i > start_cube.2) {
            break;
        }
        let pos = (start_cube.0 - i + view_point.0, start_cube.1 + i + view_point.1, start_cube.2 - i + view_point.2);
        if (get_vector_pos(pos)) < cubes.len() {
            cubes[get_vector_pos(pos)] = EMPTY_CUBE;
            return (pos.0 as usize, pos.1 as usize, pos.2 as usize);
        }
    }
    (0, 0, 0)
}

fn get_grid_pos(n: usize) -> (usize, usize, usize) {
    let x = n % VIEW_WIDTH;
    let y = (n / (VIEW_WIDTH)) % VIEW_WIDTH;
    let z = n / (VIEW_WIDTH * VIEW_WIDTH);

    (x, y, z)
}

fn get_vector_pos(world_space: (usize, usize, usize)) -> usize {
    world_space.0 + world_space.1 * GRID_WIDTH + world_space.2 * GRID_WIDTH * GRID_WIDTH
}

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

struct Agent {
    name: String,
    animation:  Animation,
    animation_state: usize,
    position: (usize, usize, usize),
}

struct Animation {
    frames: Vec<Sprite>,
    name: String
}