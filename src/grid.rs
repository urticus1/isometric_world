use std::cmp::{max};
use std::collections::{VecDeque};
use std::ops;
use std::ops::{Add, Index, IndexMut, Sub};
use std::sync::mpsc::Sender;
use crate::{Agent, GRID_HEIGHT, GRID_WIDTH};
use crate::cubes::Cube;
use crate::events::{GridChangeEvent};
use crate::render::{light_flood_fill, remove_light};
use crate::resources::CubeType;

pub struct Grid {
    pub grid: Vec<Cube>,
    width: usize,
    height: usize,
    events: Sender<GridChangeEvent>,
    active_water: VecDeque<WorldPos>,
}

impl Grid {

    pub fn handle_grid_change_event(&mut self, event: GridChangeEvent) {
        match event {
            GridChangeEvent::Delete{position, cube} => {
                if cube.cube_type.is_some() && cube.cube_type.unwrap() == CubeType::Lantern {
                    remove_light(position, 10, self);
                }
                //TODO kick off flood fill from neighbours
            }
            GridChangeEvent::MoveAgent { from, to} => {
                let agent = self.get_cube_mut(from).agent.take();
                self.get_cube_mut(to).agent = agent;
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


    fn get_vector_pos(&self, world_space: WorldPos) -> Result<usize, ()> {
        let index = world_space.x + world_space.y * self.width + world_space.z * self.width * self.width;
        if index >= self.grid.len() {
            return Err(());
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

    pub fn move_agent(&mut self, from: WorldPos, to: WorldPos) {
        let agent = self.get_cube_mut(from).agent.take();
        self.get_cube_mut(to).agent = agent;
    }

    pub fn delete_cube(&mut self, pos: WorldPos) {
        let index = self.get_vector_pos(pos).unwrap();
        let deleted = self.grid[index].clone();
        self.grid[index] = Cube::empty();
        self.events.send(GridChangeEvent::Delete {
            position: pos,
            cube: deleted
        });
    }

    pub fn place_cube(&mut self, pos: WorldPos, cube: Cube) {
        let index = self.get_vector_pos(pos).unwrap();
        if cube.cube_type == Some(CubeType::Lantern) {
            light_flood_fill(pos, 10, self);
        }
        self.grid[index] = cube;
    }

    pub fn spawn_agent(&mut self, agent: &Agent) {
        let index = self.get_vector_pos(agent.position).unwrap();
        self.grid[index] = Cube::with_agent(agent.id as u8);
    }

    pub fn get_cube(&self, pos: WorldPos) -> Result<&Cube, ()> {
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
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Light {
    pub fn new(x_level: u8, y_level: u8, z_level: u8) -> Self {
        Light {
            r: x_level,
            g: y_level,
            b: z_level,
        }
    }

    pub fn max_level() -> Light {
        Light {
            r: 220,
            g: 245,
            b: 255,
        }
    }

    pub fn min_level() -> Light {
        Light {
            r: 100,
            g: 100,
            b: 100,
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
            interpolate(min.r, max.r),
            interpolate(min.g, max.g),
            interpolate(min.b, max.b),
        )
    }
}

impl ops::Add<Light> for Light {
    type Output = Light;

    fn add(self, _rhs: Light) -> Light {
        Light {
            r: max(self.r, _rhs.r),
            g: max(self.g, _rhs.g),
            b: max(self.b, _rhs.b),
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


