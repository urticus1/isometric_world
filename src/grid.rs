use std::cmp::{max};
use std::collections::{VecDeque};
use std::ops;
use std::ops::{Add, Index, IndexMut, Sub};
use std::sync::mpsc::Sender;
use minifb::Key::G;
use crate::{Agent, GRID_HEIGHT, GRID_WIDTH};
use crate::cubes::Cube;
use crate::events::{GridChangeEvent};
use crate::render::{light_flood_fill, CubeFace};
use crate::resources::CubeType;

pub struct Grid {
    pub grid: Vec<Cube>,
    width: usize,
    height: usize,
    events: Sender<GridChangeEvent>,
    active_water: VecDeque<WorldPos>,
}

impl Grid {

    pub fn handle_grid_change_event(&mut self, event: &GridChangeEvent) {
        match event {
            GridChangeEvent::Delete(pos) => {

            }
        }
    }

    pub fn new(width: usize, height: usize, event_queue: Sender<GridChangeEvent>) -> Grid {
        Grid {
            grid: vec![Cube::new(CubeType::Stone); width * width * height],
            width: width,
            height: height,
            events: event_queue,
            active_water: VecDeque::new()
        }
    }

    pub fn is_valid_worldpos(&self, pos: &WorldPos) -> bool {
        pos.x >= 0 && pos.x <= GRID_WIDTH - 1 && pos.y >= 0 && pos.y <= GRID_WIDTH - 1 && pos.z >= 0 && pos.z <= GRID_HEIGHT - 1
    }

    pub fn get_blocking_cube(&self, face: &CubeFace, i: usize) -> Option<Cube> {
        match face {
            CubeFace::nY => {
                if (i % (self.width * self.width)) / self.width == 0 {
                    return None
                }
                Some(self.grid[i - self.width])
            },
            CubeFace::nX => {
                if i % self.width == 0 {
                    return None
                }
                Some(self.grid[i - 1])
            },
            CubeFace::pX => {
                if i % self.width == GRID_WIDTH - 1 {
                    return None
                }
                Some(self.grid[i + 1])
            },
            CubeFace::pY => {
                if (i % (self.width * self.width)) / self.width == self.width - 1 {
                    return None
                }
                Some(self.grid[i + self.width])
            },
            CubeFace::Z => {
                let val = i + self.width * self.width;
                if val >= self.height * self.width * self.width {
                    return None;
                }
                Some(self.grid[i + self.width * self.width])
            }
        }
    }

    pub fn get_cube_above(&self, i: usize) -> Option<Cube> {
        let val = i + self.width * self.width;
        if val >= self.height * self.width * self.width {
            return None;
        }
        Some(self.grid[i + self.width * self.width])
    }

    pub fn get_vector_pos(&self, world_space: WorldPos) -> Result<usize, GridError> {
        let index = world_space.x + world_space.y * self.width + world_space.z * self.width * self.width;
        if index >= self.grid.len() {
            return Err(GridError::new(format!("Index out of bounds: {}", index)));
        };
        Ok(index)
    }

    pub fn move_cube(&mut self, from: WorldPos, to: WorldPos) {
        if from == to {
            return;
        }
        let from_index = self.get_vector_pos(from).unwrap();
        let to_index = self.get_vector_pos(to).unwrap();
        let move_cube = self.grid[from_index].clone();
        self.grid[to_index] = move_cube;
        self.delete_cube(from);
    }

    pub fn delete_cube(&mut self, pos: WorldPos) {
        let index = self.get_vector_pos(pos).unwrap();
        self.grid[index] = Cube::empty();
        self.events.send(GridChangeEvent::Delete(pos));
    }

    pub fn place_cube(&mut self, pos: WorldPos, cube: Cube) {
        let index = self.get_vector_pos(pos).unwrap();
        self.grid[index] = cube;
        if cube.cube_type == Some(CubeType::Lantern) {
            light_flood_fill(pos, 10, self);
        }
    }

    pub fn spawn_agent(&mut self, agent: &Agent) {
        let index = self.get_vector_pos(agent.position).unwrap();
        self.grid[index] = Cube::with_agent(agent.id as u8);
    }

    pub fn get_cube(&self, pos: WorldPos) -> Result<&Cube, GridError> {
        Ok(&self.grid[self.get_vector_pos(pos)?])
    }

    pub fn get_cube_mut(&mut self, pos: WorldPos) -> &mut Cube {
        let index = self.get_vector_pos(pos).unwrap();
        &mut self.grid[index]
    }

    pub fn is_occupied(&self, position: WorldPos) -> bool {
        let index = self.get_vector_pos(position).unwrap();
        self.grid[index].agent.is_some() || self.grid[index].cube_type.is_some()
    }
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
            x_level: 220,
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

impl ops::Add<Light> for Light {
    type Output = Light;

    fn add(self, _rhs: Light) -> Light {
        Light {
            x_level: max(self.x_level, _rhs.x_level),
            y_level: max(self.y_level, _rhs.y_level),
            z_level: max(self.z_level, _rhs.z_level),
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct WorldPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl WorldPos {
    pub fn new(x: usize, y: usize, z: usize) -> WorldPos {
        WorldPos { x, y, z }
    }

    pub fn above(&self) -> Option<WorldPos> {
        if self.z < GRID_HEIGHT - 1 {
            return Some(WorldPos::new(self.x, self.y, self.z + 1));
        }
        None
    }

    pub fn below(&self) -> Option<WorldPos> {
        if self.z > 0 {
            return Some(WorldPos::new(self.x, self.y, self.z - 1));
        }
        None
    }

    pub fn next_x(&self) -> Option<WorldPos> {
        if self.x < GRID_WIDTH - 1 {
            return Some(WorldPos::new(self.x + 1, self.y, self.z));
        }
        None
    }

    pub fn prev_x(&self) -> Option<WorldPos> {
        if self.x > 0 {
            return Some(WorldPos::new(self.x - 1, self.y, self.z));
        }
        None
    }

    pub fn next_y(&self) -> Option<WorldPos> {
        if self.y < GRID_WIDTH - 1 {
            return Some(WorldPos::new(self.x, self.y + 1, self.z));
        }
        None
    }

    pub fn prev_y(&self) -> Option<WorldPos> {
        if self.y > 0 {
            return Some(WorldPos::new(self.x, self.y - 1, self.z));
        }
        None
    }

    pub fn horizontal_neighbours(&self) -> Vec<WorldPos> {
        [self.next_x(), self.prev_x(), self.next_y(), self.prev_y()].into_iter().flatten().collect()
    }

    pub fn face_neighbours(&self) -> Vec<WorldPos> {
        [self.next_x(), self.prev_x(), self.next_y(), self.prev_y(), self.above(), self.below()].into_iter().flatten().collect()
    }

}

impl Add<WorldPos> for WorldPos {
    type Output = WorldPos;
    fn add(self, rhs: WorldPos) -> WorldPos {
        WorldPos::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub<WorldPos> for WorldPos {
    type Output = WorldPos;

    fn sub(self, other: WorldPos) -> WorldPos {
        WorldPos::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

#[derive(Debug)]
pub struct GridError {
    message: String,
}


impl GridError {
    pub fn new(message: String) -> GridError {
        GridError {
            message: message.into(),
        }
    }
}


