use std::cmp::{max, min};
use std::collections::{HashSet, VecDeque};
use std::ops;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use minifb::Key::N;
use crate::{Agent, GRID_HEIGHT, GRID_WIDTH };
use crate::events::{EventQueue, GridChangeEvent};
use crate::render::{light_flood_fill, CubeFace};
use crate::resources::CubeType;

pub struct Grid {
    pub grid: Vec<Cube>,
    width: usize,
    height: usize,
    events: Sender<GridChangeEvent>,
    active_water: VecDeque<(usize, usize, usize)>,
}

impl Grid {

    pub fn handle_grid_change_event(&mut self, event: &GridChangeEvent) {
        match event {
            GridChangeEvent::Delete(pos) => {
                let neighbours = find_face_neighbours(*pos);
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

    pub fn get_blocking_cube(&self, face: &CubeFace, i: usize) -> Option<Cube> {
        return match face {
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

    pub fn get_vector_pos(&self, world_space: (usize, usize, usize)) -> Result<usize, String> {
        let index = world_space.0 + world_space.1 * self.width + world_space.2 * self.width * self.width;
        if index >= self.grid.len() {
            return Err(format!("Index out of bounds: {}", index));
        };
        Ok(index)
    }

    pub fn move_cube(&mut self, from: (usize, usize, usize), to: (usize, usize, usize)) {
        if from == to {
            return;
        }
        let from_index = self.get_vector_pos(from).unwrap();
        let to_index = self.get_vector_pos(to).unwrap();
        let move_cube = self.grid[from_index].clone();
        self.grid[to_index] = move_cube;
        self.delete_cube(from);
    }

    pub fn delete_cube(&mut self, pos: (usize, usize, usize)) {
        let index = self.get_vector_pos(pos).unwrap();
        let mut cube = self.grid[index];
        cube.cube_type = None;
        cube.cube_x_face = None;
        cube.cube_y_face = None;
        cube.cube_z_face = None;
        let send_event = cube.agent.is_none();
        cube.agent = None;
        self.grid[index] = cube;
        if send_event {
            self.events.send(GridChangeEvent::Delete(pos));
        }

    }

    pub fn place_cube(&mut self, pos: (usize, usize, usize), cube: Cube) {
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

    pub fn get_cube(&self, pos: (usize, usize, usize)) -> &Cube {
        &self.grid[self.get_vector_pos(pos).unwrap()]
    }

    pub fn get_cube_mut(&mut self, pos: (usize, usize, usize)) -> &mut Cube {
        let index = self.get_vector_pos(pos).unwrap();
        &mut self.grid[index]
    }

    pub fn is_occupied(&self, position: (usize, usize, usize)) -> bool {
        let index = self.get_vector_pos(position).unwrap();
        self.grid[index].agent.is_some() || self.grid[index].cube_type.is_some()
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
    pub cube_type: Option<CubeType>,
    pub cube_x_face: Option<u8>,
    pub cube_y_face: Option<u8>,
    pub cube_z_face: Option<u8>,
    pub agent: Option<u8>,
    pub light_level: Light,
    pub water_level: f32
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

impl Cube {
    pub fn new(cube_type: CubeType) -> Self {
        Cube {
            cube_type: Some(cube_type),
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }

    pub fn empty() -> Self {
        Cube {
            cube_type: None,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.cube_type != Some(CubeType::Water) && self.cube_type.is_some() && self.agent.is_none()
    }

    pub fn is_transparent(&self) -> bool {
        self.cube_type.is_none() || self.agent.is_some() || self.cube_type == Some(CubeType::Water) ||self.cube_type == Some(CubeType::Lantern)
    }

    pub fn with_agent(agent: u8) -> Cube {
        Cube {
            cube_type: None,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: Some(agent),
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }
}
