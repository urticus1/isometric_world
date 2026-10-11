use crate::{Compass};
use crate::grid::WorldPos;
use crate::render::CubeFace;

pub struct Camera {
    pub compass: Compass
}

impl Camera {

    pub fn direction(&self) -> (i8, i8, i8) {
        match self.compass {
            Compass::North => (-1, -1, -1),
            Compass::East => (-1, 1, -1),
            Compass::West => (1, 1, -1),
            Compass::South => (1, -1, -1),

        }
    }

    pub fn visible_faces(&self) -> (CubeFace, CubeFace) {
        match self.compass {
            Compass::North => (CubeFace::pY, CubeFace::pX),
            Compass::East => (CubeFace::pX, CubeFace::nY),
            Compass::South =>  (CubeFace::nY, CubeFace::nX),
            Compass::West => (CubeFace::nX, CubeFace::pY)
        }
    }

    pub fn get_obscuring_cube_left(&self, cube: WorldPos) -> Option<WorldPos> {
        match self.compass {
            Compass::North => cube.next_y(),
            Compass::East => cube.next_x(),
            Compass::South => cube.prev_y(),
            Compass::West => cube.prev_x()
        }
    }

    pub fn get_obscuring_cube_right(&self, cube: WorldPos) -> Option<WorldPos> {
        match self.compass {
            Compass::North => cube.next_x(),
            Compass::East => cube.prev_y(),
            Compass::South => cube.prev_x(),
            Compass::West => cube.next_y()
        }
    }

    pub fn get_blocked_cube_left(&self, cube: WorldPos) -> Option<WorldPos> {
        match self.compass {
            Compass::North => cube.prev_x(),
            Compass::East => cube.prev_x(),
            Compass::South => cube.next_y(),
            Compass::West => cube.next_x()
        }
    }

    pub fn get_blocked_cube_right(&self, cube: WorldPos) -> Option<WorldPos> {
        match self.compass {
            Compass::North => cube.prev_y(),
            Compass::East => cube.prev_y(),
            Compass::South => cube.next_x(),
            Compass::West => cube.prev_y()
        }
    }
}