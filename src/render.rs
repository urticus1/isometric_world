use std::collections::{HashSet, VecDeque};
use std::path::Path;
use image::open;
use crate::{SCREEN_WIDTH, TILE_WIDTH};
use crate::grid::{find_face_neighbours, Grid, Light};

const LEFT_FACE_PIXELS: [(usize, usize); 156] = [(0, 6), (1, 6), (0, 7), (1, 7), (2, 7), (3, 7), (0, 8), (1, 8), (2, 8), (3, 8), (4, 8), (5, 8), (0, 9), (1, 9), (2, 9), (3, 9), (4, 9), (5, 9), (6, 9), (7, 9), (0, 10), (1, 10), (2, 10), (3, 10), (4, 10), (5, 10), (6, 10), (7, 10), (8, 10), (9, 10), (0, 11), (1, 11), (2, 11), (3, 11), (4, 11), (5, 11), (6, 11), (7, 11), (8, 11), (9, 11), (10, 11), (11, 11), (0, 12), (1, 12), (2, 12), (3, 12), (4, 12), (5, 12), (6, 12), (7, 12), (8, 12), (9, 12), (10, 12), (11, 12), (0, 13), (1, 13), (2, 13), (3, 13), (4, 13), (5, 13), (6, 13), (7, 13), (8, 13), (9, 13), (10, 13), (11, 13), (0, 14), (1, 14), (2, 14), (3, 14), (4, 14), (5, 14), (6, 14), (7, 14), (8, 14), (9, 14), (10, 14), (11, 14), (0, 15), (1, 15), (2, 15), (3, 15), (4, 15), (5, 15), (6, 15), (7, 15), (8, 15), (9, 15), (10, 15), (11, 15), (0, 16), (1, 16), (2, 16), (3, 16), (4, 16), (5, 16), (6, 16), (7, 16), (8, 16), (9, 16), (10, 16), (11, 16), (0, 17), (1, 17), (2, 17), (3, 17), (4, 17), (5, 17), (6, 17), (7, 17), (8, 17), (9, 17), (10, 17), (11, 17), (0, 18), (1, 18), (2, 18), (3, 18), (4, 18), (5, 18), (6, 18), (7, 18), (8, 18), (9, 18), (10, 18), (11, 18), (2, 19), (3, 19), (4, 19), (5, 19), (6, 19), (7, 19), (8, 19), (9, 19), (10, 19), (11, 19), (4, 20), (5, 20), (6, 20), (7, 20), (8, 20), (9, 20), (10, 20), (11, 20), (6, 21), (7, 21), (8, 21), (9, 21), (10, 21), (11, 21), (8, 22), (9, 22), (10, 22), (11, 22), (10, 23), (11, 23)];
const RIGHT_FACE_PIXELS: [(usize, usize); 156] = [(23, 6), (22, 6), (23, 7), (22, 7), (21, 7), (20, 7), (23, 8), (22, 8), (21, 8), (20, 8), (19, 8), (18, 8), (23, 9), (22, 9), (21, 9), (20, 9), (19, 9), (18, 9), (17, 9), (16, 9), (23, 10), (22, 10), (21, 10), (20, 10), (19, 10), (18, 10), (17, 10), (16, 10), (15, 10), (14, 10), (23, 11), (22, 11), (21, 11), (20, 11), (19, 11), (18, 11), (17, 11), (16, 11), (15, 11), (14, 11), (13, 11), (12, 11), (23, 12), (22, 12), (21, 12), (20, 12), (19, 12), (18, 12), (17, 12), (16, 12), (15, 12), (14, 12), (13, 12), (12, 12), (23, 13), (22, 13), (21, 13), (20, 13), (19, 13), (18, 13), (17, 13), (16, 13), (15, 13), (14, 13), (13, 13), (12, 13), (23, 14), (22, 14), (21, 14), (20, 14), (19, 14), (18, 14), (17, 14), (16, 14), (15, 14), (14, 14), (13, 14), (12, 14), (23, 15), (22, 15), (21, 15), (20, 15), (19, 15), (18, 15), (17, 15), (16, 15), (15, 15), (14, 15), (13, 15), (12, 15), (23, 16), (22, 16), (21, 16), (20, 16), (19,16), (18, 16), (17, 16), (16, 16), (15, 16), (14, 16), (13, 16), (12, 16), (23, 17), (22, 17), (21, 17), (20, 17), (19, 17), (18, 17), (17, 17), (16, 17), (15, 17), (14, 17), (13, 17), (12, 17), (23, 18), (22, 18), (21, 18), (20, 18), (19, 18), (18, 18), (17, 18), (16, 18), (15, 18), (14, 18), (13, 18), (12, 18), (21, 19), (20, 19), (19, 19), (18, 19), (17, 19), (16, 19), (15, 19), (14, 19), (13, 19), (12, 19), (19, 20), (18, 20), (17, 20), (16, 20), (15, 20), (14, 20), (13, 20), (12, 20), (17, 21), (16, 21), (15, 21), (14, 21), (13, 21), (12, 21), (15, 22), (14, 22), (13, 22), (12, 22), (13, 23), (12, 23)];
const TOP_FACE_PIXELS: [(usize, usize); 144] = [(10, 0), (11, 0), (12, 0), (13, 0), (8, 1), (9, 1), (10, 1), (11, 1), (12, 1), (13, 1), (14, 1), (15, 1), (6, 2), (7, 2), (8, 2), (9, 2), (10, 2), (11, 2), (12, 2), (13, 2), (14, 2), (15, 2), (16, 2), (17, 2), (4, 3), (5, 3), (6, 3), (7, 3), (8, 3), (9, 3), (10, 3), (11, 3), (12, 3), (13, 3), (14, 3), (15, 3), (16, 3), (17, 3), (18, 3), (19, 3), (2, 4), (3, 4), (4, 4), (5, 4), (6, 4), (7, 4), (8, 4), (9, 4), (10, 4), (11, 4), (12, 4), (13, 4), (14, 4), (15, 4), (16, 4), (17, 4), (18, 4), (19, 4), (20, 4), (21, 4), (0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (10, 5), (11, 5), (12, 5), (13, 5), (14, 5), (15, 5), (16, 5), (17, 5), (18, 5), (19, 5), (20, 5), (21, 5), (22, 5), (23, 5), (2, 6), (3, 6), (4, 6), (5, 6), (6, 6), (7, 6), (8, 6), (9, 6), (10, 6), (11, 6), (12, 6), (13, 6), (14, 6), (15, 6), (16, 6), (17, 6), (18, 6), (19, 6), (20, 6), (21, 6), (4, 7), (5, 7), (6, 7), (7, 7), (8, 7), (9, 7), (10, 7), (11, 7), (12, 7), (13, 7), (14, 7), (15, 7), (16, 7), (17, 7), (18, 7), (19, 7), (6, 8), (7, 8), (8, 8), (9, 8), (10, 8), (11, 8), (12, 8), (13, 8), (14, 8), (15, 8), (16, 8), (17, 8), (8, 9), (9, 9), (10, 9), (11, 9), (12, 9), (13, 9), (14, 9), (15, 9), (10, 10), (11, 10), (12, 10), (13, 10)];
const ALPHA_MASK: u32 = (0b11111111u8 as u32) << 24;
const RED_MASK: u32 = (0b11111111u8 as u32) << 16;
const GREEN_MASK: u32 = (0b11111111u8 as u32) << 8;
const BLUE_MASK: u32 = (0b11111111u8 as u32);



pub struct Sprite {
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

pub fn light_flood_fill(start: (usize, usize, usize), grid: &mut Grid) {
    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();
    queue.push_back(start);

    let mut light_level = 10;
    while !queue.is_empty() && light_level > 0 {
        let mut layer = vec![];
        for i in 0..queue.len() {
            layer.push(queue.pop_front().unwrap());
        }

        for current in layer {
            seen.insert(current);
            grid.get_cube_mut(current).light_level = Light::new(light_level);

            for neighbour in find_face_neighbours(current) {
                if seen.contains(&neighbour) {
                    continue;
                }
                queue.push_back(neighbour);
            }
        }
        println!("light level: {}", light_level);
        light_level -= 1;
    }
}

pub fn draw_face(face: Face, screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>, light: (u8, u8, u8)) {
    let pixels = match face {
        Face::LEFT => LEFT_FACE_PIXELS.iter(),
        Face::RIGHT => RIGHT_FACE_PIXELS.iter(),
        Face::TOP => TOP_FACE_PIXELS.iter()
    };


    for (x,y) in pixels {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        if value & ALPHA_MASK == 0 {
            continue;
        }
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;

        let r = ((value & RED_MASK) >> 16) as u8;
        let g = ((value & GREEN_MASK) >> 8) as u8;
        let b = (value & BLUE_MASK) as u8;

        let r = r.saturating_sub(255 - light.0) as u32;
        let g = g.saturating_sub(255 - light.1) as u32;
        let b = b.saturating_sub(255 - light.2) as u32;

        let val = 0u32 | r << 16 | g << 8 | b;

        if (pixel < buffer.len()) {
            buffer[pixel] = val
        }
    }
}

pub enum Face {
    LEFT,
    RIGHT,
    TOP
}

pub fn draw_top_face(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>, highlight_colour: u32) {
    for (x,y) in TOP_FACE_PIXELS {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        if value & ALPHA_MASK == 0 {
            continue;
        }
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
        if (pixel < buffer.len()) {
            buffer[pixel] =  value.saturating_add(highlight_colour)
        }
    }
}

pub fn draw_right_face(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>, highlight_colour: u32) {
    for (x,y) in RIGHT_FACE_PIXELS {
        let value = sprite.pixels[x + y * TILE_WIDTH];
        if value & ALPHA_MASK == 0 {
            continue;
        }
        let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
        if (pixel < buffer.len()) {
            buffer[pixel] = value.saturating_add(highlight_colour)
        }
    }
}

pub fn draw_sprite(screen_pos: (usize, usize), sprite: &Sprite, buffer: &mut Vec<u32>) {
    for x in 0..TILE_WIDTH {
        for y in 0..TILE_WIDTH {
            let value = sprite.pixels[x + y * TILE_WIDTH];
            if value & (0b11111111u8 as u32) << 24 == 0 {
                continue;
            }
            let pixel = (screen_pos.0 + SCREEN_WIDTH * screen_pos.1) + x + y * SCREEN_WIDTH;
            if (pixel < buffer.len()) {
                buffer[pixel] = value
            }
        }
    }
}