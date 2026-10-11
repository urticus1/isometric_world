use crate::grid::{Light};
use crate::resources::CubeType;

#[derive(Copy, Clone)]
pub struct Cube {
    pub cube_type: Option<CubeType>,
    pub cube_px_face: Option<u8>,
    pub cube_nx_face: Option<u8>,
    pub cube_py_face: Option<u8>,
    pub cube_ny_face: Option<u8>,
    pub cube_z_face: Option<u8>,
    pub agent: Option<u8>,
    pub light_level: Light,
    pub water_level: f32
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
            light_level: Light::min_level(),
            water_level: 0.0
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
            cube_py_face: None,
            cube_ny_face: None,
            cube_px_face: None,
            cube_nx_face: None,
            cube_z_face: None,
            agent: Some(agent),
            light_level: Light::min_level(),
            water_level: 0.0
        }
    }
}