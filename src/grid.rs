use std::ops::{Index, IndexMut};
use crate::{Agent, EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH, VIEW_WIDTH};



pub struct Grid {
    pub grid: Vec<Cube>,
}

impl Grid {

    pub fn new(x: usize, y: usize, height: usize) -> Grid {
        Grid {
            grid: vec![Cube::new(0); GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH]
        }
    }

    pub fn get_cube_next_x(&self, i: usize) -> Option<Cube> {
        if i % GRID_WIDTH == GRID_WIDTH -1 {
            return None
        }
        Some(self.grid[i + 1])
    }

    pub fn get_cube_next_y(&self, i: usize) -> Option<Cube> {
        if i % (GRID_WIDTH * GRID_WIDTH) + GRID_WIDTH >= GRID_WIDTH * GRID_WIDTH {
            return None
        }
        Some(self.grid[i + GRID_WIDTH])
    }

    pub fn get_cube_above(&self, i: usize) -> Option<Cube> {
        let val = i + GRID_WIDTH * GRID_WIDTH;
        if val >= GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH {
            return None;
        }
        Some(self.grid[i + GRID_WIDTH * GRID_WIDTH])
    }

    pub fn get_grid_pos(&self, n: usize) -> (usize, usize, usize) {
        let x = n % VIEW_WIDTH;
        let y = (n / (VIEW_WIDTH)) % VIEW_WIDTH;
        let z = n / (VIEW_WIDTH * VIEW_WIDTH);

        (x, y, z)
    }

    pub fn get_vector_pos(&self, world_space: (usize, usize, usize)) -> usize {
        world_space.0 + world_space.1 * GRID_WIDTH + world_space.2 * GRID_WIDTH * GRID_WIDTH
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
}


impl Cube {
    pub fn new(cube_type: u8) -> Self {
        Cube {
            cube_type: cube_type,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: None,
        }
    }

    pub fn is_walkable(&self) -> bool {
        self.cube_type == EMPTY_CUBE
    }

    pub fn is_transparent(&self) -> bool {
        self.cube_type == EMPTY_CUBE || self.agent.is_some()
    }

    pub fn with_agent(agent: u8) -> Cube {
        Cube {
            cube_type: EMPTY_CUBE,
            cube_y_face: None,
            cube_x_face: None,
            cube_z_face: None,
            agent: Some(agent),
        }
    }
}
