use std::path::Path;
use image::open;
use minifb::{Key, MouseMode, Window, WindowOptions};

const SCREEN_WIDTH: usize = 2000;
const SCREEN_HEIGHT: usize = 1200;
const SCREEN_Y_OFFSET: usize = SCREEN_HEIGHT / 4;

const TILE_WIDTH: usize = 24;
const TILE_HALF_WIDTH: usize = TILE_WIDTH / 2;

const GRID_HEIGHT: usize = 120;
const GRID_WIDTH: usize = 120;

const VIEW_HEIGHT: usize = 20;
const VIEW_WIDTH: usize = 60;

const CUBE_TYPE_MASK: u32 = 0b11111111u8 as u32;
const RIGHT_FACE_CUBE_MASK: u32 = CUBE_TYPE_MASK << 8;
const LEFT_FACE_CUBE_MASK: u32 = CUBE_TYPE_MASK << 16;
const TOP_FACE_CUBE_MASK: u32 = CUBE_TYPE_MASK << 24;

const EMPTY_CUBE: u32 = 8;

fn main() {
    let stone = Sprite::new("resources/24/stone.png");
    let mud = Sprite::new("resources/24/mud.png");
    let grass = Sprite::new("resources/24/grass.png");
    let blank = Sprite::new("resources/24/blank.png");
    let floor = Sprite::new("resources/24/floor.png");
    let man = Sprite::new("resources/24/man1.png");

    let sprites = vec![stone, mud, grass, blank, floor, man];

    let mut cubes = vec![0u32; GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH];

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_WIDTH {

            cubes[get_vector_pos((i,GRID_HEIGHT - 1,j))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,GRID_HEIGHT - 2,j))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,GRID_HEIGHT - 3,j))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,GRID_HEIGHT - 4,j))] = EMPTY_CUBE;
            cubes[get_vector_pos((i,GRID_HEIGHT - 5,j))] = 2 + (4 << 24);
            cubes[get_vector_pos((i,GRID_HEIGHT - 6,j))] = 2;
            cubes[get_vector_pos((i,GRID_HEIGHT - 7,j))] = 1;
            cubes[get_vector_pos((i,GRID_HEIGHT - 8,j))] = 1;
        }
    }

    cubes[get_vector_pos((GRID_WIDTH-1, GRID_HEIGHT - 1,GRID_WIDTH-1))] =5;

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

    let mut window = Window::new(
        "Cubes",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("{}", e));

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = vec![0xFFFFFF; SCREEN_WIDTH * SCREEN_HEIGHT];


    let mut view_x = GRID_WIDTH - VIEW_WIDTH;
    let mut view_z = GRID_WIDTH - VIEW_WIDTH;
    let mut view_y = GRID_HEIGHT - VIEW_HEIGHT;
    while window.is_open() && !window.is_key_down(Key::Escape) {

        let mut buffer = buffer.clone();
        if window.get_keys().contains(&Key::Right) && view_x < GRID_WIDTH - VIEW_WIDTH {
            view_x += 1;
        }
        if window.get_keys().contains(&Key::Left) && view_x > 0 {
            view_x -= 1;
        }
        if window.get_keys().contains(&Key::W) && view_z < GRID_WIDTH - VIEW_WIDTH {
            view_z += 1;
        }
        if window.get_keys().contains(&Key::S) && view_z > 0 {
            view_z -= 1;
        }
        if window.get_keys().contains(&Key::Down) && view_y > 0 {
            view_y -= 1;
        }
        if window.get_keys().contains(&Key::Up) && view_y < GRID_HEIGHT - VIEW_HEIGHT {
            view_y += 1;
        }

        if let Some((sx, sy)) = window.get_mouse_pos(MouseMode::Clamp) {
            select_cube((sx as i32, sy as i32), &mut cubes, (view_x, view_y, view_z));
        }

        for y in 0..VIEW_HEIGHT {
            for z in 0..VIEW_WIDTH {
                for x in 0..VIEW_WIDTH {
                    let view_index = x + y * VIEW_WIDTH * VIEW_WIDTH + z * VIEW_WIDTH;
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH * GRID_WIDTH) + (z + view_z) * GRID_HEIGHT;


                    let cube_data = cubes[cube_index];
                    let cube_type = cube_data & CUBE_TYPE_MASK;
                    if (cube_type == EMPTY_CUBE) {
                        continue;
                    }

                    let (cube_screen_x, cube_screen_y) = get_screen_coord(get_grid_pos(view_index));


                    if let Some(next_x) = get_cube_next_x(cube_index) {

                        if cubes[next_x] & CUBE_TYPE_MASK == EMPTY_CUBE {
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


                    if let Some(next_z) = get_cube_next_z(cube_index) {
                        if cubes[next_z] & CUBE_TYPE_MASK == EMPTY_CUBE {
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
                        if cubes[next_y] & CUBE_TYPE_MASK == EMPTY_CUBE {
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

fn get_cube_next_z(i: usize) -> Option<usize> {
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

fn select_cube(screen_space: (i32, i32), cubes: &mut Vec<u32>, view_point: (usize, usize, usize)) -> (usize, usize, usize) {

    let sx = screen_space.0 - SCREEN_WIDTH as i32 / 2;
    let sy = screen_space.1 - SCREEN_HEIGHT as i32 / 2;

    let a = sx / TILE_WIDTH as i32;
    let b = sy / TILE_HALF_WIDTH as i32;
    let z = view_point.2 as i32;

    let x = (a + b + 2 * z) / 2;
    let y = (b - a + 2 * z) / 2;

    if (x < 0 || y < 0) {
        return (0, 0, 0);
    }
    let start_cube = (x as usize, y as usize, z as usize);
    //println!("{:?}", start_cube);
    for i in 0..5 {
        if (i > start_cube.0 || i > start_cube.1 || i > start_cube.2) {
            break;
        }
        let pos = (start_cube.0 - i + view_point.0, start_cube.1 - i + view_point.1, start_cube.2 - i + view_point.2);
        if (get_vector_pos(pos)) < cubes.len() {
            println!("{:?}", pos);
            cubes[get_vector_pos(pos)] = EMPTY_CUBE;
            return (pos.0 as usize, pos.1 as usize, pos.2 as usize);
        }
        //println!("{:?}", pos);
    }
    (0, 0, 0)
}

fn get_grid_pos(n: usize) -> (usize, usize, usize) {
    let x = n % VIEW_WIDTH;
    let y = n / (VIEW_WIDTH * VIEW_WIDTH);
    let z = (n / (VIEW_WIDTH)) % VIEW_WIDTH;

    (x, y, z)
}

fn get_vector_pos(world_space: (usize, usize, usize)) -> usize {
    world_space.0 + world_space.1 * GRID_WIDTH * GRID_WIDTH + world_space.2 * GRID_WIDTH
}

struct Sprite {
    pub pixels: [u32; TILE_WIDTH * TILE_WIDTH],
}

fn draw_sprite(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    for x in 0..TILE_WIDTH {
        for y in 0..TILE_WIDTH {
            let value = sprite.pixels[x + y * TILE_WIDTH];
            //println!("{} == {}", value, 16777216u32);
            //println!("{}", value == 16777216u32);
           // println!("x{} y{}", x, y);
            if  value == 18816799 {
                continue
            }
          //  println!("didnt skip");
            let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
            if (pixel < buffer.len()) {
                buffer[pixel] = value
            }
        }
    }
}

fn draw_left_face(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    let values = vec![(0, 6), (1, 6), (0, 7), (1, 7), (2, 7), (3, 7), (0, 8), (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (0, 9), (1, 9), (2, 9), (3, 9), (4, 9), (5, 9), (6, 9), (7, 9), (0, 10), (1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10), (0, 11), (1, 11), (2, 11), (3, 11), (4, 11), (5, 11), (6, 11), (7, 11), (8, 11), (9, 11), (10, 11), (11, 11), (0, 12), (1, 12), (2, 12), (3, 12), (4, 12), (5, 12), (6, 12), (7, 12), (8, 12), (9, 12), (10, 12), (11, 12), (0, 13), (1, 13), (2, 13), (3, 13), (4, 13), (5, 13), (6, 13), (7, 13), (8, 13), (9, 13), (10, 13), (11, 13), (0, 14), (1, 14), (2, 14), (3, 14), (4, 14), (5, 14), (6, 14), (7, 14), (8, 14), (9, 14), (10, 14), (11, 14), (0, 15), (1, 15), (2, 15), (3, 15), (4, 15), (5, 15), (6, 15), (7, 15), (8, 15), (9, 15), (10, 15), (11, 15), (0, 16), (1, 16), (2, 16), (3, 16), (4, 16), (5, 16), (6, 16), (7, 16), (8, 16), (9, 16), (10, 16), (11, 16), (0, 17), (1, 17), (2, 17), (3, 17), (4, 17), (5, 17), (6, 17), (7, 17), (8, 17), (9, 17), (10, 17), (11, 17), (0, 18), (1, 18), (2, 18), (3, 18), (4, 18), (5, 18), (6, 18), (7, 18), (8, 18), (9, 18), (10, 18), (11, 18),  (2, 19), (3, 19), (4, 19), (5, 19), (6, 19), (7, 19), (8, 19), (9, 19), (10, 19), (11, 19),  (4, 20), (5, 20), (6, 20), (7, 20), (8, 20), (9, 20), (10, 20), (11, 20),  (6, 21), (7, 21), (8, 21), (9, 21), (10, 21), (11, 21),  (8, 22), (9, 22), (10, 22), (11, 22),  (10, 23), (11, 23)];

    for (x,y) in values {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        //println!("{} == {}", value, 16777216u32);
        //println!("{}", value == 16777216u32);
        // println!("x{} y{}", x, y);
        if  value == 18816799 {
            continue
        }
        //  println!("didnt skip");
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
        if (pixel < buffer.len()) {
            buffer[pixel] = value
        }
    }
}

fn draw_top_face(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    let values = vec![(10, 0), (11, 0), (12, 0), (13, 0), (8, 1), (9, 1), (10, 1), (11, 1), (12, 1), (13, 1), (14, 1), (15, 1), (6, 2), (7, 2), (8, 2), (9, 2), (10, 2), (11, 2), (12, 2), (13, 2), (14, 2), (15, 2), (16, 2), (17, 2), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3), (9, 3), (10, 3), (11, 3), (12, 3), (13, 3), (14, 3), (15, 3), (16, 3), (17, 3), (18, 3), (19, 3), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), (8, 4), (9, 4), (10, 4), (11, 4), (12, 4), (13, 4), (14, 4), (15, 4), (16, 4), (17, 4), (18, 4), (19, 4), (20, 4), (21, 4), (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5), (13, 5), (14, 5), (15, 5), (16, 5), (17, 5), (18, 5), (19, 5), (20, 5), (21, 5), (22, 5), (23, 5), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), (8, 6), (9, 6), (10, 6), (11, 6), (12, 6), (13, 6), (14, 6), (15, 6), (16, 6), (17, 6), (18, 6), (19, 6), (20, 6), (21, 6), (4, 7), (5, 7), (6, 7), (7, 7), (8, 7), (9, 7), (10, 7), (11, 7), (12, 7), (13, 7), (14, 7), (15, 7), (16, 7), (17, 7), (18, 7), (19, 7), (6, 8), (7, 8), (8, 8), (9, 8), (10, 8), (11, 8), (12, 8), (13, 8), (14, 8), (15, 8), (16, 8), (17, 8), (8, 9), (9, 9), (10, 9), (11, 9), (12, 9), (13, 9), (14, 9), (15, 9), (10, 10), (11, 10), (12, 10), (13, 10)];
      for (x,y) in values {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        //println!("{} == {}", value, 16777216u32);
        //println!("{}", value == 16777216u32);
        // println!("x{} y{}", x, y);
        if  value == 18816799 {
            continue
        }
        //  println!("didnt skip");
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
        if (pixel < buffer.len()) {
            buffer[pixel] = value
        }
    }
}

fn draw_right_face(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    let values = vec![(23, 6), (22, 6), (23, 7), (22, 7), (21, 7), (20, 7), (23, 8), (22, 8), (21, 8), (20, 8), (19, 8), (18, 8), (23, 9), (22, 9), (21, 9), (20, 9), (19, 9), (18, 9), (17, 9), (16, 9), (23, 10), (22, 10), (21, 10), (20, 10), (19, 10), (18, 10), (17, 10), (16, 10), (15, 10), (14, 10), (23, 11), (22, 11), (21, 11), (20, 11), (19, 11), (18, 11), (17, 11), (16, 11), (15, 11), (14, 11), (13, 11), (12, 11), (23, 12), (22, 12), (21, 12), (20, 12), (19, 12), (18, 12), (17, 12), (16, 12), (15, 12), (14, 12), (13, 12), (12, 12), (23, 13), (22, 13), (21, 13), (20, 13), (19, 13), (18, 13), (17, 13), (16, 13), (15, 13), (14, 13), (13, 13), (12, 13), (23, 14), (22, 14), (21, 14), (20, 14), (19, 14), (18, 14), (17, 14), (16, 14), (15, 14), (14, 14), (13, 14), (12, 14), (23, 15), (22, 15), (21, 15), (20, 15), (19, 15), (18, 15), (17, 15), (16, 15), (15, 15), (14, 15), (13, 15), (12, 15), (23, 16), (22, 16), (21, 16), (20, 16), (19,
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               16), (18, 16), (17, 16), (16, 16), (15, 16), (14, 16), (13, 16), (12, 16), (23, 17), (22, 17), (21, 17), (20, 17), (19, 17), (18, 17), (17, 17), (16, 17), (15, 17), (14, 17), (13, 17), (12, 17), (23, 18), (22, 18), (21, 18), (20, 18), (19, 18), (18, 18), (17, 18), (16, 18), (15, 18), (14, 18), (13, 18), (12, 18), (21, 19), (20, 19), (19, 19), (18, 19), (17, 19), (16, 19), (15, 19), (14, 19), (13, 19), (12, 19), (19, 20), (18, 20), (17, 20), (16, 20), (15, 20), (14, 20), (13, 20), (12, 20), (17, 21), (16, 21), (15, 21), (14, 21), (13, 21), (12, 21), (15, 22), (14, 22), (13, 22), (12, 22), (13, 23), (12, 23)];
    for (x,y) in values {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        //println!("{} == {}", value, 16777216u32);
        //println!("{}", value == 16777216u32);
        // println!("x{} y{}", x, y);
        if  value == 18816799 {
            continue
        }
        //  println!("didnt skip");
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
        if (pixel < buffer.len()) {
            buffer[pixel] = value
        }
    }
}

impl Sprite {
    pub fn new(img: &str) -> Self {
        let img = open(Path::new(img)).expect(&format!("Error loading sprite {}", img)).into_rgb8();
        let mut pixels = [0u32; TILE_WIDTH * TILE_WIDTH];
        for (i, pixel) in img.pixels().enumerate() {
            let val = 1 << 24 | (pixel.0[0] as u32) << 16 | (pixel.0[1] as u32) << 8 | pixel.0[2] as u32;
            pixels[i] = val
        }

        Sprite {
            pixels
        }
    }
}