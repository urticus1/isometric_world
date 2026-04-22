use std::fmt::format;
use std::path::Path;
use image::open;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 2000;
const HEIGHT: usize = 1200;

const TILE_HEIGHT: usize = 8;

const TILE_WIDTH: usize = 16;

const GRID_HEIGHT: usize = 60;
const GRID_WIDTH: usize = 60;

const VIEW_HEIGHT: usize = 30;
const VIEW_WIDTH: usize = 50;

fn main() {
    let stone = Sprite::new("resources/stone.png");
    let mud = Sprite::new("resources/mud.png");
    let grass = Sprite::new("resources/grass.png");

    let sprites = vec![stone, mud, grass];

    let mut cubes = vec![0u32; GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH];

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_WIDTH {
            cubes[get_vector_pos((i,GRID_HEIGHT - 2,j))] = 1;
            cubes[get_vector_pos((i,GRID_HEIGHT - 3,j))] = 1;
            cubes[get_vector_pos((i,GRID_HEIGHT - 4,j))] = 1;
        }
    }

    let epicentre= (GRID_WIDTH / 2, GRID_HEIGHT -1, GRID_WIDTH / 2);

    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            for k in 0..GRID_WIDTH {
                if (i as i32 - epicentre.0 as i32).pow(2) + (j as i32 - epicentre.1 as i32).pow(2) + (k as i32 - epicentre.2 as i32).pow(2) < 30 {
                    cubes[get_vector_pos((i,j,k))] = 5;
                }
            }
        }
    }

    let mut window = Window::new(
        "Minifb Example",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("{}", e));

    // Limit to ~60 fps
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = vec![0xFFFFFF; WIDTH * HEIGHT];

    let mut view_x = 0;
    let mut view_z = 0;
    let mut view_y = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0xFFFFFF);

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

        for y in 0..VIEW_HEIGHT {
            for z in 0..VIEW_WIDTH {
                for x in 0..VIEW_WIDTH {
                    let view_index = x + y * VIEW_WIDTH * VIEW_WIDTH + z * VIEW_WIDTH;
                    let cube_index = (x + view_x) + ((y + view_y) * GRID_WIDTH * GRID_WIDTH) + (z + view_z) * GRID_HEIGHT;

                    if (cubes[cube_index] == 5) {
                        continue;
                    }

                    if let Some(next_x) = get_cube_next_x(cube_index) {
                        if let Some(next_z) = get_cube_next_z(cube_index) {
                            if let Some(above) = get_cube_above(cube_index) {
                                if cubes[next_x] != 5 && cubes[next_z] != 5 && cubes[above] != 5 {
                                    continue
                                }
                            }
                        }
                    }

                    let cube = get_screen_coord(get_grid_pos(view_index));
                    let cube = (cube.0 + (WIDTH / 2) as i32, cube.1 + (HEIGHT / 2) as i32);
                    draw_sprite((cube.0 as usize - 16, cube.1 as usize - 32), &sprites[cubes[cube_index] as usize], &mut buffer);
                }
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn get_cube_next_x(i: usize) -> Option<usize> {
    if i % GRID_WIDTH == GRID_WIDTH -1 {
        return None
    }
    return Some(i + 1)
}

fn get_cube_next_z(i: usize) -> Option<usize> {
    if i % (GRID_WIDTH * GRID_WIDTH) + GRID_WIDTH >= GRID_WIDTH * GRID_WIDTH {
        return None
    }
    return Some(i + GRID_WIDTH)
}

fn get_cube_above(i: usize) -> Option<usize> {
    let val = i + GRID_WIDTH * GRID_WIDTH;
    if val >= GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH {
        return None;
    }
    Some(i + GRID_WIDTH * GRID_WIDTH)
}

fn get_screen_coord(world_space: (usize, usize, usize)) -> (i32, i32) {
    let x = world_space.0 as i32;
    let y = world_space.1 as i32;
    let z = world_space.2 as i32;

    (
        (x - y) * TILE_WIDTH as i32,
        (x + y - 2 * z) * TILE_HEIGHT as i32,
    )
}

fn get_grid_pos(n: usize) -> (usize, usize, usize) {
    let x = n % VIEW_WIDTH;
    let y = (n / (VIEW_WIDTH)) % VIEW_WIDTH;
    let z = n / (VIEW_WIDTH * VIEW_WIDTH);

    (x, y, z)
}

fn get_vector_pos(world_space: (usize, usize, usize)) -> usize {
    world_space.0 + world_space.1 * GRID_WIDTH * GRID_WIDTH + world_space.2 * GRID_WIDTH
}

struct Sprite {
    pub pixels: [u32; 32 * 32],
}

fn draw_sprite(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    for x in 0..32 {
        for y in 0..32 {
            if sprite.pixels[x + y * 32] == 16777216 {
                continue
            }
            let pixel = (screen_pos.0 + WIDTH * screen_pos.1) + x + y * WIDTH;
            if (pixel < buffer.len()) {
                buffer[pixel] = sprite.pixels[x + y * 32]
            }
        }
    }

}

impl Sprite {
    pub fn new(img: &str) -> Self {
        let img = open(Path::new(img)).expect(&format!("Error loading sprite {}", img)).into_rgb8();
        let mut pixels = [0u32; 32 * 32];
        for (i, pixel) in img.pixels().enumerate() {
            let val = 1 << 24 | (pixel.0[0] as u32) << 16 | (pixel.0[1] as u32) << 8 | pixel.0[2] as u32;
            pixels[i] = val
        }

        Sprite {
            pixels
        }
    }
}