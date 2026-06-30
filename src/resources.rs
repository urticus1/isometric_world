use std::collections::HashMap;
use std::sync::Arc;
use crate::animation::{Animation, AnimationPool};
use crate::animation::Animation::{CameraFacingAnimation, CompassAwareAnimation};
use crate::render::Sprite;

pub fn load_animations() -> AnimationPool {
    let man1 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing1.png");
    let man2 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing2.png");
    let man3 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing3.png");
    let man4 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing4.png");
    let man5 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing5.png");
    let man6 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing6.png");
    let man7 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing7.png");
    let man8 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing8.png");
    let man9 = Sprite::new("resources/24/man/animations/ploughing/man_ploughing9.png");

    let man_mining1 = Sprite::new("resources/24/man/animations/mining/man_mining1.png");
    let man_mining2 = Sprite::new("resources/24/man/animations/mining/man_mining2.png");
    let man_mining3 = Sprite::new("resources/24/man/animations/mining/man_mining3.png");
    let man_mining4 = Sprite::new("resources/24/man/animations/mining/man_mining4.png");
    let man_mining5 = Sprite::new("resources/24/man/animations/mining/man_mining5.png");
    let man_mining6 = Sprite::new("resources/24/man/animations/mining/man_mining6.png");
    let man_mining7 = Sprite::new("resources/24/man/animations/mining/man_mining7.png");
    let man_mining8 = Sprite::new("resources/24/man/animations/mining/man_mining8.png");
    let man_mining9 = Sprite::new("resources/24/man/animations/mining/man_mining9.png");
    let man_mining10 = Sprite::new("resources/24/man/animations/mining/man_mining10.png");
    let man_mining11 = Sprite::new("resources/24/man/animations/mining/man_mining11.png");

    let man_r_ne1 = Sprite::new("resources/24/man/animations/running/man_running_ne1.png");
    let man_r_ne2 = Sprite::new("resources/24/man/animations/running/man_running_ne2.png");
    let man_r_ne3 = Sprite::new("resources/24/man/animations/running/man_running_ne3.png");
    let man_r_ne4 = Sprite::new("resources/24/man/animations/running/man_running_ne4.png");
    let man_r_ne5 = Sprite::new("resources/24/man/animations/running/man_running_ne5.png");
    let man_r_ne6 = Sprite::new("resources/24/man/animations/running/man_running_ne6.png");

    let man_r_nw1 = Sprite::new("resources/24/man/animations/running/man_running_nw1.png");
    let man_r_nw2 = Sprite::new("resources/24/man/animations/running/man_running_nw2.png");
    let man_r_nw3 = Sprite::new("resources/24/man/animations/running/man_running_nw3.png");
    let man_r_nw4 = Sprite::new("resources/24/man/animations/running/man_running_nw4.png");
    let man_r_nw5 = Sprite::new("resources/24/man/animations/running/man_running_nw5.png");
    let man_r_nw6 = Sprite::new("resources/24/man/animations/running/man_running_nw6.png");

    let man_r_sw1 = Sprite::new("resources/24/man/animations/running/man_running_sw1.png");
    let man_r_sw2 = Sprite::new("resources/24/man/animations/running/man_running_sw2.png");
    let man_r_sw3 = Sprite::new("resources/24/man/animations/running/man_running_sw3.png");
    let man_r_sw4 = Sprite::new("resources/24/man/animations/running/man_running_sw4.png");
    let man_r_sw5 = Sprite::new("resources/24/man/animations/running/man_running_sw5.png");
    let man_r_sw6 = Sprite::new("resources/24/man/animations/running/man_running_sw6.png");

    let man_r_se1 = Sprite::new("resources/24/man/animations/running/man_running_se1.png");
    let man_r_se2 = Sprite::new("resources/24/man/animations/running/man_running_se2.png");
    let man_r_se3 = Sprite::new("resources/24/man/animations/running/man_running_se3.png");
    let man_r_se4 = Sprite::new("resources/24/man/animations/running/man_running_se4.png");
    let man_r_se5 = Sprite::new("resources/24/man/animations/running/man_running_se5.png");
    let man_r_se6 = Sprite::new("resources/24/man/animations/running/man_running_se6.png");

    let man_idle = Sprite::new("resources/24/man/man_idle.png");
    let plough_animation = Arc::new(CameraFacingAnimation {
        frames: vec![man1, man2, man3, man4, man5, man6, man7, man8, man9],
        name: "ploughing".to_string(),
    });

    let mining_animation = Arc::new(CameraFacingAnimation {
        frames: vec![man_mining1, man_mining2, man_mining3, man_mining4, man_mining5, man_mining6, man_mining7, man_mining8, man_mining9, man_mining10, man_mining11],
        name: "mining".to_string(),
    });

    let idle_animation = Arc::new(CameraFacingAnimation {
        frames: vec![man_idle],
        name: "idle".to_string(),
    });

    let run_animation = Arc::new(CompassAwareAnimation {
        frames_ne: vec![man_r_ne1, man_r_ne2, man_r_ne3, man_r_ne4, man_r_ne5, man_r_ne6],
        frames_nw: vec![man_r_nw1, man_r_nw2, man_r_nw3, man_r_nw4, man_r_nw5, man_r_nw6],
        frames_sw: vec![man_r_sw1, man_r_sw2, man_r_sw3, man_r_sw4, man_r_sw5, man_r_sw6],
        frames_se: vec![man_r_se1, man_r_se2, man_r_se3, man_r_se4, man_r_se5, man_r_se6],
        name: "running".to_string(),
    });

    AnimationPool {
        animations: HashMap::from([
            ("ploughing".to_string(), plough_animation),
            ("mining".to_string(), mining_animation),
            ("running".to_string(), run_animation),
            ("idle".to_string(), idle_animation),
        ])
    }
}

pub fn load_cube_sprites() -> Vec<Sprite> {
    let stone = Sprite::new("resources/24/stone_no_light.png");
    let mud = Sprite::new("resources/24/mud_no_light.png");
    let grass = Sprite::new("resources/24/grass_no_light.png");
    let blank = Sprite::new("resources/24/blank.png");
    let floor = Sprite::new("resources/24/floor.png");
    let man = Sprite::new("resources/24/man/man_idle.png");

    let water = Sprite::new("resources/24/water.png");
    let lantern = Sprite::new("resources/24/lantern.png");
    let water_half = Sprite::new("resources/24/water_half.png");
    let water_quarter = Sprite::new("resources/24/water_quarter.png");
    let water_small = Sprite::new("resources/24/water_small.png");

    vec![stone, mud, grass, blank, floor, man, water, lantern, water_half, water_quarter, water_small]
}