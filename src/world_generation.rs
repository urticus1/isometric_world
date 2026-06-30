use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use image::{open, RgbaImage};
use rand::random_range;
use crate::events::GridChangeEvent;
use crate::grid::{Cube, Grid};
use crate::{EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH, WATER_CUBE};
use crate::agents::Agent;
use crate::agents::Direction::Px;
use crate::render::CubeFace::pX;
use crate::resources::load_animations;

struct PerlinNoise {
    pub pixels: RgbaImage,
    pub z_scale: f32,
}

impl PerlinNoise {
    pub fn get_noise(&self, x: usize, y: usize) -> f32 {
        let width = self.pixels.width();
        let height = self.pixels.height();
        let x = x % width as usize;
        let y = y % height as usize;
        let val = self.pixels.get_pixel(x as u32, y as u32);
        val.0[0] as f32 * self.z_scale
    }
}

pub fn prepare_grid(events: Sender<GridChangeEvent>) -> Grid {
    let mut grid = Grid::new(GRID_WIDTH, GRID_HEIGHT, events);

    let perlin = PerlinNoise {
        pixels: open(Path::new("resources/perlin_greyscale.png")).unwrap().into_rgba8(),
        z_scale: 0.07,
    };
    let ground_level = GRID_HEIGHT - 20;
    let frequency_x: f32 = 0.2;
    let variance_x = 5.0;
    let frequency_y = 0.1;
    let variance_y = 6.0;
    let sea_level = ground_level - 20;


    for x in 0..GRID_WIDTH {
        for y in 0..GRID_WIDTH {
            for z in 0..GRID_HEIGHT {
                let val = perlin.get_noise(x, y);
                let cut_off= ground_level as f32 + val; // = ground_level as f32
                //+ (x as f32 * frequency_x).sin() * variance_x + (x as f32 * frequency_x * 4.0).sin() * variance_x / 8.0
                //+ (y as f32 * frequency_y).sin() * variance_y+ (y as f32 * frequency_y * 4.0).sin() * variance_y / 8.0;
                let cut_off = cut_off as usize;
                let coord = (x,y,z);
                let index = grid.get_vector_pos(coord).unwrap();
                if z > cut_off {
                    if z > sea_level {
                        grid[index] = Cube::new(EMPTY_CUBE);
                    }
                    else {
                        grid[index] = Cube::new(WATER_CUBE);
                        grid[index].water_level = 100.0;
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

    for _ in 0..20 {
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

pub fn place_workers(grid: &mut Grid) -> Vec<Agent> {
    let man_x = GRID_WIDTH - 40;
    let man_y = GRID_WIDTH - 30;
    let man2_x = GRID_WIDTH - 35;
    let man2_y = GRID_WIDTH - 31;

    let (man_z, man2_z) = {
        (find_ground_spawn_z(grid, man_x, man_y), find_ground_spawn_z(grid, man2_x, man2_y))
    };

    let worker_animations = Arc::new(load_animations());
    let mut man = Agent {
        animation: worker_animations.animations["idle"].clone(),
        position: (man_x, man_y, man_z),
        animation_pool: Arc::clone(&worker_animations),
        name: "man".to_string(),
        animation_state: 0,
        destination: None,
        tasks: vec![],
        active_task: None,
        id: 0,
        direction: Px
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
        id: 1,
        direction: Px
    };

    grid.spawn_agent(&mut man);
    grid.spawn_agent(&mut man2);

    vec![man, man2]
}

fn find_ground_spawn_z(grid: &Grid, x: usize, y: usize) -> usize {
    for z in (0..GRID_HEIGHT).rev() {
        if grid.get_cube((x, y, z)).cube_type != EMPTY_CUBE {
            return (z + 1).min(GRID_HEIGHT - 1);
        }
    }
    0
}