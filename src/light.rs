use std::cmp::{max,};
use std::collections::{HashSet, VecDeque};
use crate::grid::{Grid, WorldPos};
use crate::render::Face;

const MAX_LIGHT_LEVEL: u8 = 15;

#[derive(Copy, Clone)]
pub struct Light {
    pub colour: Colour
}

#[derive(Copy, Clone)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Colour {
        Colour { r, g, b }
    }
}

pub struct AmbientLight {
    pub left_colour: Colour,
    pub right_colour: Colour,
    pub top_colour: Colour,
}

impl AmbientLight {
    pub fn new(left_colour: Colour, right_colour: Colour, top_colour: Colour) -> AmbientLight {
        AmbientLight {
            left_colour,
            right_colour,
            top_colour,
        }
    }

    pub fn get_face_light(&self, face: Face) -> Light {
        match face {
            Face::Left => Light::from_colour(self.left_colour),
            Face::Right => Light::from_colour(self.right_colour),
            Face::Top => Light::from_colour(self.top_colour),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LightLevel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn light_flood_fill(start: WorldPos, grid: &mut Grid) {
    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();
    queue.push_back(start);
    seen.insert(start);

    let mut light_level = grid.get_cube(start).unwrap().light_level;
    if light_level.is_none() {
        return;
    }


    let mut light_level = light_level.unwrap();
    while !queue.is_empty() && !light_level.is_min_level() {
        let layer_size = queue.len();
        for _ in 0..layer_size {
            let current = queue.pop_front().unwrap();
            let cube = grid.get_cube_mut(current);

            //opaque cubes have None light
            if let Some(cube_light) = cube.light_level {
                cube.light_level = Some(cube_light.mix(&light_level));
            }

            for neighbour in current.face_neighbours() {
                if !seen.contains(&neighbour) {
                    seen.insert(neighbour);
                    if let Ok(c) = grid.get_cube(neighbour) {
                        if c.is_transparent() {
                            queue.push_back(neighbour);
                        }
                    }
                }
            }



        }
        light_level = light_level.dim();
    }
}

pub fn remove_light(start: WorldPos, light_level: LightLevel, grid: &mut Grid) {
    let mut queue = VecDeque::new();
    let mut refill_queue: VecDeque<WorldPos> = VecDeque::new();
    let mut seen = HashSet::new();
    queue.push_back(start);
    seen.insert(start);

    let mut light_level = light_level;

    while !queue.is_empty() && !light_level.is_min_level() {
        let layer_size = queue.len();
        for _ in 0..layer_size {
            let current = queue.pop_front().unwrap();
            let cube = grid.get_cube_mut(current);

            if !cube.is_transparent() {
                continue;
            }

            if let Some(cube_light) = cube.light_level {
                if cube_light.r > light_level.r {
                    refill_queue.push_back(current);
                    continue;
                }
            }

            cube.light_level = Some(LightLevel::min_level());
            for neighbour in current.face_neighbours() {
                if !seen.contains(&neighbour) {
                    seen.insert(neighbour);
                    queue.push_back(neighbour);
                }
            }
        }
        light_level = light_level.dim();
    }

    for light_source in refill_queue {
        light_flood_fill(light_source, grid);
    }
}

impl LightLevel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        LightLevel {
            r,g,b
        }
    }

    pub fn max_level() -> Self {
        LightLevel {
            r: MAX_LIGHT_LEVEL,
            g: MAX_LIGHT_LEVEL,
            b: MAX_LIGHT_LEVEL,
        }
    }

    pub fn min_level() -> Self {
        LightLevel {
            r: 0,
            g: 0,
            b: 0,
        }
    }

    pub fn dim(&self) -> LightLevel {
        LightLevel {
            r: (self.r - 1).max(0),
            g: (self.g - 1).max(0),
            b: (self.b - 1).max(0),
        }
    }

    pub fn is_min_level(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    pub fn mix(&self, other: &LightLevel) -> LightLevel {
        LightLevel {
            r: max(self.r, other.r),
            g: max(self.g, other.g),
            b: max(self.b, other.b),
        }
    }
}

impl Light {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Light {
            colour: Colour::new(r, g, b),
        }
    }

    pub fn from_colour(colour: Colour) -> Self {
        Light {
            colour,
        }
    }

    pub fn from_level(level: LightLevel) -> Light {
        let min = 100;
        let max = 255;

        let interpolate = |t| -> u8 {
            (min as f32 + ((max - min) as f32 * (t as f32 / MAX_LIGHT_LEVEL as f32))).round() as u8
        };

        Light::new(
            interpolate(level.r),
            interpolate(level.g),
            interpolate(level.b),
        )
    }

    pub fn add(&self, other: Light) -> Self {
        let colour = Colour {
            r: self.colour.r.saturating_add(other.colour.r),
            g: self.colour.g.saturating_add(other.colour.g),
            b: self.colour.b.saturating_add(other.colour.b),


        };
        Light {
            colour: colour,
        }
    }

    pub fn add_if_present(&self, other: Option<Light>) -> Self {
        match other {
            None => self.clone(),
            Some(other) => self.add(other),
        }
    }
}