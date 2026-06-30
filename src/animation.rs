use std::collections::HashMap;
use std::sync::Arc;
use crate::Sprite;


pub struct AnimationPool {
    pub animations: HashMap<String, Arc<Animation>>,
}



pub enum Animation {
    CompassAwareAnimation {
        frames_ne: Vec<Sprite>,
        frames_se: Vec<Sprite>,
        frames_sw: Vec<Sprite>,
        frames_nw: Vec<Sprite>,
        name: String
    },
    CameraFacingAnimation {
        frames: Vec<Sprite>,
        name: String
    }
}

impl Animation {
    pub fn name(&self) -> &String {
        match self {
            Animation::CompassAwareAnimation { name, .. } => name,
            Animation::CameraFacingAnimation { name, .. } => name
        }
    }

    pub fn length(&self) -> usize {
        match self {
            Animation::CompassAwareAnimation {frames_ne, ..} => frames_ne.len(),
            Animation::CameraFacingAnimation {frames, ..} => frames.len()
        }
    }
}