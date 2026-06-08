use crate::{EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH, VIEW_WIDTH};

pub fn get_cube_next_x(i: usize) -> Option<usize> {
    if i % GRID_WIDTH == GRID_WIDTH -1 {
        return None
    }
    Some(i + 1)
}

pub fn get_cube_next_y(i: usize) -> Option<usize> {
    if i % (GRID_WIDTH * GRID_WIDTH) + GRID_WIDTH >= GRID_WIDTH * GRID_WIDTH {
        return None
    }
    Some(i + GRID_WIDTH)
}

pub fn get_cube_above(i: usize) -> Option<usize> {
    let val = i + GRID_WIDTH * GRID_WIDTH;
    if val >= GRID_HEIGHT * GRID_WIDTH * GRID_WIDTH {
        return None;
    }
    Some(i + GRID_WIDTH * GRID_WIDTH)
}

pub fn get_grid_pos(n: usize) -> (usize, usize, usize) {
    let x = n % VIEW_WIDTH;
    let y = (n / (VIEW_WIDTH)) % VIEW_WIDTH;
    let z = n / (VIEW_WIDTH * VIEW_WIDTH);

    (x, y, z)
}

pub fn get_vector_pos(world_space: (usize, usize, usize)) -> usize {
    world_space.0 + world_space.1 * GRID_WIDTH + world_space.2 * GRID_WIDTH * GRID_WIDTH
}

pub fn move_cube(grid: &mut Vec<u64>, from: (usize, usize, usize), to: (usize, usize, usize)) {
    if from == to {
        return;
    }
    grid[get_vector_pos(to)] = grid[get_vector_pos(from)];
    grid[get_vector_pos(from)] = EMPTY_CUBE;
}