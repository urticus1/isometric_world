use crate::light::{LightLevel};
use crate::render::CubeFace;
use crate::resources::CubeType;

#[derive(Clone)]
pub struct Cube {
    pub cube_type: Option<CubeType>,
    pub cube_px_face: Option<u8>,
    pub cube_nx_face: Option<u8>,
    pub cube_py_face: Option<u8>,
    pub cube_ny_face: Option<u8>,
    pub cube_z_face: Option<u8>,
    pub agent: Option<u8>,
    pub light_level: Option<LightLevel>,
}

impl Cube {
    pub fn new(cube_type: CubeType) -> Self {
        Cube {
            cube_type: Some(cube_type),
            cube_py_face: None,
            cube_ny_face: None,
            cube_px_face: None,
            cube_nx_face: None,
            cube_z_face: None,
            agent: None,
            light_level: None,
        }
    }

    pub fn light_source(cube_type: CubeType, light_level: LightLevel) -> Self {
        Cube {
            cube_type: Some(cube_type),
            cube_py_face: None,
            cube_ny_face: None,
            cube_px_face: None,
            cube_nx_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Some(light_level),
        }
    }

    pub fn empty() -> Self {
        Cube {
            cube_type: None,
            cube_py_face: None,
            cube_ny_face: None,
            cube_px_face: None,
            cube_nx_face: None,
            cube_z_face: None,
            agent: None,
            light_level: Some(LightLevel::min_level()),
        }
    }

    pub fn get_face_sprite_index(&self, visible_face: CubeFace) -> Option<u8> {
        if let Some(cube_type) = self.cube_type {
            return match visible_face {
                CubeFace::PosX => Some(self.cube_px_face.unwrap_or(cube_type as u8)),
                CubeFace::PosY => Some(self.cube_py_face.unwrap_or(cube_type as u8)),
                CubeFace::NegX => Some(self.cube_nx_face.unwrap_or(cube_type as u8)),
                CubeFace::NegY => Some(self.cube_ny_face.unwrap_or(cube_type as u8)),
                CubeFace::Z => Some(self.cube_z_face.unwrap_or(cube_type as u8)),
            }
        }
        None
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
            cube_py_face: None,
            cube_ny_face: None,
            cube_px_face: None,
            cube_nx_face: None,
            cube_z_face: None,
            agent: Some(agent),
            light_level: None,
        }
    }
}