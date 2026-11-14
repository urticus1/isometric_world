use crate::{Compass};
use crate::light::{AmbientLight, Colour};
use crate::render::{CubeFace, Face};

const TOP_FACE_AMBIENT_LIGHT: Colour = Colour { r: 250 , g: 250 , b: 250 };
const LEFT_FACE_AMBIENT_LIGHT: Colour = Colour { r: 220 , g: 220 , b: 220 };
const RIGHT_FACE_AMBIENT_LIGHT: Colour = Colour { r: 200 , g: 200 , b: 200 };

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
            Compass::North => (CubeFace::PosY, CubeFace::PosX),
            Compass::East => (CubeFace::PosX, CubeFace::NegY),
            Compass::South =>  (CubeFace::NegY, CubeFace::NegX),
            Compass::West => (CubeFace::NegX, CubeFace::PosY)
        }
    }

    pub fn get_ambient_light(&self) -> AmbientLight {
        AmbientLight {
            left_colour: LEFT_FACE_AMBIENT_LIGHT,
            right_colour: RIGHT_FACE_AMBIENT_LIGHT,
            top_colour: TOP_FACE_AMBIENT_LIGHT,
        }
    }
}