use std::ops::{Index, IndexMut};
use crate::{Agent, EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH, VIEW_WIDTH, WATER_CUBE};



pub struct Grid {
    pub grid: Vec<Cube>,
    width: usize,
    height: usize
}

impl Grid {

    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            grid: vec![Cube::new(0); width * width * height],
            width: width,
            height: height
        }
    }

    pub fn get_cube_next_x(&self, i: usize) -> Option<Cube> {
        if i % self.width == self.width -1 {
            return None
        }
        Some(self.grid[i + 1])
    }

    pub fn get_cube_next_y(&self, i: usize) -> Option<Cube> {
        if i % (self.width * self.width) + self.width >= self.width * self.width {
            return None
        }
        Some(self.grid[i + self.width])
    }

    pub fn get_cube_above(&self, i: usize) -> Option<Cube> {
        let val = i + self.width * self.width;
        if val >= self.height * self.width * self.width {
            return None;
        }
        Some(self.grid[i + self.width * self.width])
    }

    pub fn get_vector_pos(&self, world_space: (usize, usize, usize)) -> usize {
        world_space.0 + world_space.1 * self.width + world_space.2 * self.width * self.width
    }

    pub fn move_cube(&mut self, from: (usize, usize, usize), to: (usize, usize, usize)) {
        if from == to {
            return;
        }
        let from_index = self.get_vector_pos(from);
        let to_index = self.get_vector_pos(to);
        let move_cube = self.grid[from_index].clone();
        self.grid[to_index] = move_cube;
        self.grid[from_index] = Cube::new(EMPTY_CUBE);
    }

    pub fn spawn_agent(&mut self, agent: &Agent) {
        let index = self.get_vector_pos(agent.position);
        self.grid[index] = Cube::with_agent(agent.id as u8);
    }

    pub fn get_cube(&self, pos: (usize, usize, usize)) -> &Cube {
        &self.grid[self.get_vector_pos(pos)]
    }

    pub fn get_cube_mut(&mut self, pos: (usize, usize, usize)) -> &mut Cube {
        let index = self.get_vector_pos(pos);
        &mut self.grid[index]
    }

    pub fn is_occupied(&self, position: (usize, usize, usize)) -> bool {
        let index = self.get_vector_pos(position);
        self.grid[index].agent.is_some() || self.grid[index].cube_type != EMPTY_CUBE
    }
}

pub fn find_horizontal_neighbours(start: (usize, usize, usize)) -> Vec<(usize, usize, usize)> {
    let mut result = Vec::new();
    for dir in [(-1 as isize, 0 as isize), (1, 0), (0, -1), (0, 1)].iter() {
        let neighbour = (start.0 as isize + dir.0, start.1 as isize + dir.1, start.2);
        if neighbour.0 < 0 || neighbour.0 > (GRID_WIDTH - 1) as isize || neighbour.1 < 0 || neighbour.1 > (GRID_WIDTH - 1) as isize {
            continue;
        }
        let neighbour = (neighbour.0 as usize, neighbour.1 as usize, neighbour.2 as usize);
        result.push(neighbour);
    }
    result
}

pub fn find_face_neighbours(start: (usize, usize, usize)) -> Vec<(usize, usize, usize)> {
    let mut result = Vec::new();
    for dir in [(-1 as isize, 0 as isize, 0 as isize), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0,0,1)].iter() {
        let neighbour = (start.0 as isize + dir.0, start.1 as isize + dir.1, start.2 as isize + dir.2);
        if neighbour.0 < 0 || neighbour.0 > (GRID_WIDTH - 1) as isize || neighbour.1 < 0 || neighbour.1 > (GRID_WIDTH - 1) as isize || neighbour.2 < 0 || neighbour.2 > (GRID_HEIGHT - 1) as isize {
            continue;
        }
        let neighbour = (neighbour.0 as usize, neighbour.1 as usize, neighbour.2 as usize);
        result.push(neighbour);
    }
    result
}

pub fn get_manhattan_distance(start: (usize, usize, usize), end: (usize, usize, usize)) -> usize {
    (end.0 - start.0) + (end.1 - start.1) + (end.2 - start.2)
}

pub fn is_horizontal_neighbour(start: (usize, usize, usize), end: (usize, usize, usize)) -> bool {
    start.2 == end.2 && (end.0 as isize - start.0 as isize).abs() <= 1 && (end.1 as isize - start.1 as isize).abs() <= 1
}

impl Index<usize> for Grid {
    type Output = Cube;

    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index]
    }
}

impl IndexMut<usize> for Grid {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.grid[index]
    }
}

#[derive(Copy, Clone)]
pub struct Cube {
    pub cube_type: u8,
    pub cube_x_face: Option<u8>,
    pub cube_y_face: Option<u8>,
    pub cube_z_face: Option<u8>,
    pub agent: Option<u8>,
    pub light_level: Light,
}

#[derive(Copy, Clone)]
pub struct Light {
    pub x_level: u8,
    pub y_level: u8,
    pub z_level: u8,
}

impl Light {
    pub fn new(x_level: u8, y_level: u8, z_level: u8) -> Self {
        Light {
            x_level,
            y_level,
            z_level,
        }
    }

    pub fn max_level() -> Light {
        Light {
            x_level: 235,
            y_level: 245,
            z_level: 255,
        }
    }

    pub fn min_level() -> Light {
        Light {
            x_level: 100,
            y_level: 100,
            z_level: 100,
        }
    }

    pub fn from_level(level: u8) -> Light {
        const MAX_LEVEL: u8 = 10;
        let level = level.min(MAX_LEVEL);
        let min = Light::min_level();
        let max = Light::max_level();

        let interpolate = |min_v: u8, max_v: u8| -> u8 {
            min_v + ((max_v - min_v) as u32 * level as u32 / MAX_LEVEL as u32) as u8
        };

        Light::new(
            interpolate(min.x_level, max.x_level),
            interpolate(min.y_level, max.y_level),
            interpolate(min.z_level, max.z_level),
        )
    }
}

impl Cube {
    pub fn new(cube_type: u8) -> Self {
        Cube {
            cube_type: cube_type,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Light::min_level()
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.cube_type != WATER_CUBE && self.cube_type != EMPTY_CUBE
    }

    pub fn is_transparent(&self) -> bool {
        self.cube_type == EMPTY_CUBE || self.agent.is_some() || self.cube_type == WATER_CUBE
    }

    pub fn with_agent(agent: u8) -> Cube {
        Cube {
            cube_type: EMPTY_CUBE,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: Some(agent),
            light_level: Light::min_level()
        }
    }
}
