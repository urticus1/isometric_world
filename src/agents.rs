use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use crate::{Compass, Sprite};
use crate::animation::{Animation, AnimationPool};
use crate::cubes::Cube;
use crate::grid::{Grid, WorldPos};

pub fn find_path(start: WorldPos, end: WorldPos, grid: &Grid) -> Option<Vec<WorldPos>> {
    println!("{:?} -> {:?}", start, end);
    let mut closed: HashMap<WorldPos, Rc<Node>> = HashMap::new();
    let mut nodes: HashMap<WorldPos, Rc<Node>> = HashMap::new();

    let start = Rc::new(Node {
        h: 0.0,
        f: 0.0,
        g: 0.0,
        parent: None,
        position: start
    });

    let mut open: BinaryHeap<Rc<Node>> = BinaryHeap::new();
    open.push(start);

    while !open.is_empty() {
        let current = open.pop().unwrap();
        //path.push(current.position);
        for neighbour in get_walkable_neighbours(current.position, grid) {
            if closed.contains_key(&neighbour) {
                continue;
            }
            let g = current.g + 1.0;
            let h = distance(neighbour, end);
            let f = h + g;
            if !nodes.contains_key(&neighbour) {
                let node = Rc::from(Node {
                    h: distance(neighbour, end),
                    f: distance(neighbour, end) + current.g + 1.0,
                    g: current.g + 1.0,
                    parent: Some(Rc::clone(&current)),
                    position: neighbour
                });
                open.push(Rc::clone(&node));
                nodes.insert(neighbour, Rc::clone(&node));
            }
            else {
                let node = nodes.get_mut(&neighbour).unwrap();
                if f < node.f {
                    let node = Rc::from(Node {
                        h: distance(neighbour, end),
                        f: distance(neighbour, end) + current.g + 1.0,
                        g: current.g + 1.0,
                        parent: Some(Rc::clone(&current)),
                        position: neighbour
                    });
                    open.push(Rc::clone(&node));
                }
            }
            if neighbour == end {
                return Some(reconstruct_path(Rc::clone(&nodes[&end])));
            }
        }
        closed.insert(current.position, current);

    }

    println!("no path found");
    None
}

fn reconstruct_path(end: Rc<Node>) -> Vec<WorldPos> {
    let mut path = Vec::new();
    let mut current = Some(end);

    while let Some(node) = current {
        path.push(node.position);
        current = node.parent.clone();
    }
    path.remove(path.len() - 1);
    path
}

fn distance(start: WorldPos, end: WorldPos) -> f32 {
    let x = (end.x as i32 - start.x as i32 ).pow(2) as f32;
    let y = (end.y as i32 - start.y as i32 ).pow(2) as f32;
    let z = (end.z as i32 - start.z as i32 ).pow(2) as f32;
    (x + y + z).sqrt()
}

struct Node {
    h: f32,
    f: f32,
    g: f32,
    parent: Option<Rc<Node>>,
    position: WorldPos
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.total_cmp(&self.f)
            .then_with(|| other.h.total_cmp(&self.h))

    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.parent == other.parent
    }
}

impl Eq for Node {}

fn get_walkable_neighbours(position: WorldPos, grid: &Grid) -> Vec<WorldPos> {
    [position.next_x(), position.prev_x(), position.next_y(), position.prev_y()]
        .into_iter()
        .flatten()
        .map(|pos| (pos, grid.get_cube(pos).unwrap()))
        .filter(|(pos, cube)| cube.agent.is_none())
        .flat_map(|(pos, cube)| {
            match cube.cube_type {
                Some(_) => {
                    if let Some(above) = pos.above() {
                        let cube_above = grid.get_cube(above).unwrap();
                        if cube.is_walkable() && cube_above.cube_type.is_none() && cube_above.agent.is_none() {
                            return Some(above);
                        }
                    }

                },
                None => {
                    let below = pos.below();
                    if let Some(below) = below {
                        let cube_below = grid.get_cube(below).unwrap();
                        if cube_below.is_walkable() {
                            return Some(pos);
                        }
                        else if let Some(below_below) = below.below() {
                            let cube_below_below = grid.get_cube(below_below).unwrap();
                            if cube_below_below.is_walkable() && cube_below.agent.is_none() {
                                return Some(below);
                            }
                        }
                    }
                }
            }
            None
        })
        .collect()
}

pub enum AgentEvent {
    AgentAddTask {
        task: AgentTask,
        agent: usize
    }
}

pub enum Direction {
    Px,
    Py,
    Nx,
    Ny
}

pub struct Agent {
    pub name: String,
    pub animation: Arc<Animation>,
    pub animation_state: usize,
    pub animation_pool: Arc<AnimationPool>,
    pub position: WorldPos,
    pub destination: Option<WorldPos>,
    pub tasks: Vec<AgentTask>,
    pub active_task: Option<AgentCoroutine>,
    pub id: usize,
    pub direction: Direction,
}

impl Agent {

    pub fn get_current_animation_frame(&self, compass: Compass) -> &Sprite {
        match &*self.animation {
            Animation::CameraFacingAnimation {frames, ..} => &frames[self.animation_state],
            Animation::CompassAwareAnimation {frames_ne, frames_nw, frames_se, frames_sw, ..} => {
                match (compass, &self.direction) {
                    (Compass::North, Direction::Px) => &frames_se[self.animation_state],
                    (Compass::North, Direction::Py) => &frames_sw[self.animation_state],
                    (Compass::North, Direction::Nx) => &frames_nw[self.animation_state],
                    (Compass::North, Direction::Ny) => &frames_ne[self.animation_state],

                    (Compass::East, Direction::Px) => &frames_sw[self.animation_state],
                    (Compass::East, Direction::Py) => &frames_nw[self.animation_state],
                    (Compass::East, Direction::Nx) => &frames_ne[self.animation_state],
                    (Compass::East, Direction::Ny) => &frames_se[self.animation_state],

                    (Compass::South, Direction::Px) => &frames_nw[self.animation_state],
                    (Compass::South, Direction::Py) => &frames_ne[self.animation_state],
                    (Compass::South, Direction::Nx) => &frames_se[self.animation_state],
                    (Compass::South, Direction::Ny) => &frames_sw[self.animation_state],

                    (Compass::West, Direction::Px) => &frames_ne[self.animation_state],
                    (Compass::West, Direction::Py) => &frames_se[self.animation_state],
                    (Compass::West, Direction::Nx) => &frames_sw[self.animation_state],
                    (Compass::West, Direction::Ny) => &frames_nw[self.animation_state],
                }
            }
        }
    }

    pub fn change_animation(&mut self, animation: &str) {
        if self.animation.name().as_str() == animation {
            return;
        }
        self.animation_state = 0;
        let animations = Arc::clone(&self.animation_pool);
        self.animation = Arc::clone(animations.as_ref().animations.get(animation).expect(format!("Failed to get animation {}", animation).as_str()));
    }

    pub fn advance_animation_state(&mut self)  {
        self.animation_state += 1;
        if self.animation_state >= self.animation.length() {
            self.animation_state = 0;
        }
    }
}

pub struct AgentCoroutine {
    pub end_tick: u32,
    pub task: AgentTask,
    pub completed: bool,
}

#[derive(Clone)]
pub enum AgentTask {
    Move {
        path: Vec<WorldPos>,
        destination: WorldPos,
    },
    FindPath {
        destination: WorldPos,
    },
    Plough {
        target: WorldPos
    },
    Dig {
        target: WorldPos,
    },
    Place {
        target: WorldPos,
        cube: Cube
    }
}


impl AgentTask {

    /**
    * return the positions required by the agent in order to perform this task
    */
    pub fn get_required_position(&self) -> Option<Vec<WorldPos>> {
        match self {
            AgentTask::Move { .. } => None,
            AgentTask::FindPath { .. } => None,
            AgentTask::Plough { target } => Some(vec![WorldPos::new(target.x, target.y, target.z + 1)]),
            AgentTask::Dig { target } => Some(WorldPos::new(target.x, target.y, target.z + 1).horizontal_neighbours()),
            AgentTask::Place { target, .. } => Some(WorldPos::new(target.x, target.y, target.z).horizontal_neighbours()),
        }
    }
}