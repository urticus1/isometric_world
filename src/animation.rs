use std::collections::HashMap;
use std::sync::Arc;
use crate::Sprite;


pub struct AnimationPool {
    pub animations: HashMap<String, Arc<Animation>>,
}

pub struct Animation {
    pub frames: Vec<Sprite>,
    pub name: String
}