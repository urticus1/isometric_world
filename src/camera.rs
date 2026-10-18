use crate::{Compass};
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

}