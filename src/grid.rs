use std::cmp::{max, min};
use std::collections::{HashSet, VecDeque};
use std::ops;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::sync::mpsc::Sender;
use minifb::Key::N;
use crate::{Agent, Compass, EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH, LANTERN_CUBE, VIEW_WIDTH, WATER_CUBE};
use crate::events::{EventQueue, GridChangeEvent};
use crate::render::{light_flood_fill, FaceReal};

pub struct Grid {
    pub grid: Vec<Cube>,
    width: usize,
    height: usize,
    events: Sender<GridChangeEvent>,
    active_water: VecDeque<(usize, usize, usize)>,
}

impl Grid {

    pub fn update_active_water(&mut self) {
        let length = self.active_water.len();
        for i in 0..length {
            let water = self.active_water.pop_front().unwrap();
            let below = (water.0, water.1, water.2 - 1);
            let cube_below = self.get_cube(below);
            if cube_below.cube_type == EMPTY_CUBE {
                self.move_cube(water, below);
                self.active_water.push_back(below);
            }
            else if cube_below.cube_type == WATER_CUBE && cube_below.water_level != 100.0 {
                let water_below = cube_below.water_level;
                let diff = 100.0 - water_below;
                let water_above = self.get_cube_mut(water);
                if water_above.water_level > diff {
                    water_above.water_level -= diff;
                    self.get_cube_mut(below).water_level += diff;
                    self.active_water.push_back(water);
                }
                else {
                    self.get_cube_mut(below).water_level += water_above.water_level;
                    self.delete_cube(water);
                }
            }
            else {
                let mut changed_cubes = vec![];

                let original_water = self.get_cube(water);
                let mut volume_available = original_water.water_level;
                for neighbour in find_horizontal_neighbours(water) {
                    let neighbour_cube = self.get_cube(neighbour);
                    if !(neighbour_cube.cube_type == WATER_CUBE || neighbour_cube.cube_type == EMPTY_CUBE) {
                        continue;
                    }
                    if neighbour_cube.water_level < original_water.water_level - 0.1 || neighbour_cube.water_level > original_water.water_level + 0.1 {
                        changed_cubes.push(neighbour);
                        volume_available += neighbour_cube.water_level;
                    }
                }
                if (changed_cubes.is_empty()) {
                    return;
                }
                let volume = volume_available / changed_cubes.len() as f32;

                if original_water.water_level < volume - 0.1 || original_water.water_level > volume + 0.1 {
                    changed_cubes.push(water);
                }
                for cube in changed_cubes {
                    if volume > 1.0 {
                        self.get_cube_mut(cube).water_level = volume;
                        self.get_cube_mut(cube).cube_type = WATER_CUBE;
                        self.active_water.push_back(cube);
                    }
                    else {
                        self.delete_cube(cube);
                    }
                }
            }
        }
    }

    /**
    pub fn update_active_water(&mut self) {
        let length = self.active_water.len();
        for i in 0..length {
            let water = self.active_water.pop().unwrap();
            let below = (water.0, water.1, water.2 - 1);
            let cube_below = self.get_cube(below);
            if cube_below.cube_type == EMPTY_CUBE {
                self.move_cube(water, below);
                self.active_water.push(below);
            }
            else if cube_below.cube_type == WATER_CUBE {
                let water_below = cube_below.water_level;
                let diff = 100.0 - water_below;
                let water_above = self.get_cube_mut(water);
                if water_above.water_level > diff {
                    water_above.water_level -= diff;
                    self.get_cube_mut(below).water_level += diff;
                    self.active_water.push(water);
                }
                else {
                    self.get_cube_mut(below).water_level += water_above.water_level;
                    self.delete_cube(water);
                }
            }
            else {
                let mut water_in_layer = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back(water);
                water_in_layer.insert(water);

                let mut volume = 0.0;
                while !queue.is_empty() {
                    let current = queue.pop_front().unwrap();
                    let current_data = self.get_cube(current);
                    volume += current_data.water_level;

                    for neighbour in find_horizontal_neighbours(current) {
                        let data = self.get_cube(neighbour);
                        if water_in_layer.contains(&neighbour) || ! (data.cube_type == WATER_CUBE || data.cube_type == EMPTY_CUBE) {
                            continue;
                        }
                        water_in_layer.insert(neighbour);
                        queue.push_back(neighbour);
                    }
                }
                volume = volume / water_in_layer.len() as f32;
                for cube in water_in_layer {
                    if volume > 1.0 {
                        self.get_cube_mut(cube).water_level = volume;
                        self.get_cube_mut(cube).cube_type = WATER_CUBE;
                    }
                    else {
                        self.delete_cube(cube);
                    }
                }
            }
        }
    }

   */

    pub fn handle_grid_change_event(&mut self, event: &GridChangeEvent) {
        match event {
            GridChangeEvent::Delete(pos) => {
                let neighbours = find_face_neighbours(*pos);
                for neighbour in neighbours {
                    if self.get_cube(neighbour).cube_type == WATER_CUBE {
                        self.active_water.push_back(neighbour);
                    }
                }
            }
        }
    }

    pub fn new(width: usize, height: usize, event_queue: Sender<GridChangeEvent>) -> Grid {
        Grid {
            grid: vec![Cube::new(0); width * width * height],
            width: width,
            height: height,
            events: event_queue,
            active_water: VecDeque::new()
        }
    }

    pub fn get_blocking_cube_x(&self, compass: &Compass, i: usize) -> Option<Cube> {
        return match compass {
            Compass::North => {
                if i % self.width == self.width -1 {
                    return None
                }
                Some(self.grid[i + 1])
            },
            Compass::East => {
                if i % self.width == self.width -1 {
                    return None
                }
                Some(self.grid[i + 1])
            },
            Compass::South => {
                if i % self.width == 0 {
                    return None
                }
                Some(self.grid[i + 1])
            },
            Compass::West => {
                if i % self.width == 0 {
                    return None
                }
                Some(self.grid[i - 1])
            },
            _ => None
        }
    }

    pub fn get_blocking_cube(&self, face: &FaceReal, i: usize) -> Option<Cube> {
        return match face {
            FaceReal::nY => {
                if (i % (self.width * self.width)) / self.width == 0 {
                    return None
                }
                Some(self.grid[i - self.width])
            },
            FaceReal::nX => {
                if i % self.width == 0 {
                    return None
                }
                Some(self.grid[i - 1])
            },
            FaceReal::pX => {
                if i % self.width == GRID_WIDTH - 1 {
                    return None
                }
                Some(self.grid[i + 1])
            },
            FaceReal::pY => {
                if (i % (self.width * self.width)) / self.width == self.width - 1 {
                    return None
                }
                Some(self.grid[i + self.width])
            },
            FaceReal::Z => {
                let val = i + self.width * self.width;
                if val >= self.height * self.width * self.width {
                    return None;
                }
                Some(self.grid[i + self.width * self.width])
            }
        }

    }

    pub fn get_blocking_cube_y(&self, compass: &Compass, i: usize) -> Option<Cube> {
        return match compass {
            Compass::North => {
                if i % (self.width * self.width) + self.width >= self.width * self.width {
                    return None
                }
                Some(self.grid[i + self.width])
            },
            Compass::West => {
                if i % (self.width * self.width) + self.width >= self.width * self.width {
                    return None
                }
                Some(self.grid[i + self.width])
            },
            Compass::East => {
                if i % (self.width * self.width) + self.width < self.width {
                    return None
                }
                Some(self.grid[i - self.width])
            },
            Compass::South => {
                if i % (self.width * self.width) + self.width < self.width {
                    return None
                }
                Some(self.grid[i + self.width])
            },
            _ => None
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
        cube.cube_type = EMPTY_CUBE;
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
        if cube.cube_type == LANTERN_CUBE {
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
    pub fn new(cube_type: u8) -> Self {
        Cube {
            cube_type: cube_type,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.cube_type != WATER_CUBE && self.cube_type != EMPTY_CUBE && self.agent.is_none()
    }

    pub fn is_transparent(&self) -> bool {
        self.cube_type == EMPTY_CUBE || self.agent.is_some() || self.cube_type == WATER_CUBE ||self.cube_type == LANTERN_CUBE
    }

    pub fn with_agent(agent: u8) -> Cube {
        Cube {
            cube_type: EMPTY_CUBE,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: Some(agent),
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }
}
