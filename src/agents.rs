use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use crate::{Sprite, EMPTY_CUBE, GRID_HEIGHT, GRID_WIDTH};
use crate::animation::{Animation, AnimationPool};
use crate::grid::{find_horizontal_neighbours, Cube, Grid};

pub fn find_path(start: (usize, usize, usize), end: (usize, usize, usize), grid: &Grid) -> Option<Vec<(usize, usize, usize)>> {
    println!("{:?} -> {:?}", start, end);
    let mut closed: HashMap<(usize, usize, usize), Rc<Node>> = HashMap::new();
    let mut nodes: HashMap<(usize, usize, usize), Rc<Node>> = HashMap::new();

    let start = Rc::new(Node {
        h: 0,
        f: 0,
        g: 0,
        parent: None,
        position: start
    });

    let mut open: BinaryHeap<Rc<Node>> = BinaryHeap::new();
    open.push(start);

    while !open.is_empty() {
        let current = open.pop().unwrap();
        //path.push(current.position);
        for neighbour in get_neighbours(current.position, grid) {
            if closed.contains_key(&neighbour) {
                continue;
            }
            let g = current.g + 1;
            let h = distance(neighbour, end);
            let f = h + g;
            if !nodes.contains_key(&neighbour) {
                let node = Rc::from(Node {
                    h: distance(neighbour, end),
                    f: distance(neighbour, end) + current.g + 1,
                    g: current.g + 1,
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
                        f: distance(neighbour, end) + current.g + 1,
                        g: current.g + 1,
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

    None
}

fn reconstruct_path(end: Rc<Node>) -> Vec<(usize, usize, usize)> {
    let mut path = Vec::new();
    let mut current = Some(end);

    while let Some(node) = current {
        path.push(node.position);
        current = node.parent.clone();
    }
    path.remove(path.len() - 1);
    path
}

fn distance(start: (usize, usize, usize), end: (usize, usize, usize)) -> u32 {
    let x = (end.0 as i32 - start.0 as i32 ).pow(2) as u32;
    let y = (end.1 as i32 - start.1 as i32 ).pow(2) as u32;
    let z = (end.2 as i32 - start.2 as i32 ).pow(2) as u32;
    x + y + z
}

#[derive(Eq, PartialEq, Clone)]
struct Node {
    h: u32,
    f: u32,
    g: u32,
    parent: Option<Rc<Node>>,
    position: (usize, usize, usize)
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f)
            .then_with(|| other.h.cmp(&self.h))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_neighbours(position: (usize, usize, usize), grid: &Grid) -> Vec<(usize, usize, usize)> {
    let mut neighbours: Vec<(usize, usize, usize)> = Vec::new();

    for dir in [(-1 as i32, 0 as i32), (1, 0), (0, -1), (0, 1)].iter() {
        let next = (position.0 as i32 + dir.0, position.1 as i32 + dir.1, position.2);
        if next.0 < 0 || next.0 > GRID_WIDTH as i32 - 1 || next.1 < 0 || next.1 > GRID_WIDTH as i32 - 1 {
            continue;
        }
        let next = (next.0 as usize, next.1 as usize, next.2 as usize);
        if grid[grid.get_vector_pos(next).unwrap()].cube_type == EMPTY_CUBE {
            if grid[grid.get_vector_pos((next.0, next.1, next.2 - 1)).unwrap()].is_walkable() {
                neighbours.push(next);
            }
            else if
                position.2 >= 2
                && grid[grid.get_vector_pos((next.0, next.1, next.2 - 1)).unwrap()].cube_type == EMPTY_CUBE
                && grid[grid.get_vector_pos((next.0, next.1, next.2 - 2)).unwrap()].is_walkable() {
                    neighbours.push((next.0, next.1, next.2 - 1));
            }
        }
        else if position.2 < GRID_HEIGHT - 1 && grid[grid.get_vector_pos((next.0, next.1, next.2 + 1)).unwrap()].cube_type == EMPTY_CUBE && grid[grid.get_vector_pos(next).unwrap()].is_walkable(){
            neighbours.push((next.0, next.1, next.2 + 1));
        }
    }
    neighbours
}

pub enum AgentEvent {
    AgentAddTask {
        task: AgentTask,
        agent: usize
    }
}


pub struct Agent {
    pub name: String,
    pub animation: Arc<Animation>,
    pub animation_state: usize,
    pub animation_pool: Arc<AnimationPool>,
    pub position: (usize, usize, usize),
    pub destination: Option<(usize, usize, usize)>,
    pub tasks: Vec<AgentTask>,
    pub active_task: Option<AgentCoroutine>,
    pub id: usize,
}

impl Agent {
    pub fn change_animation(&mut self, animation: &str) {
        if self.animation.name == animation {
            return;
        }
        self.animation_state = 0;
        let animations = Arc::clone(&self.animation_pool);
        self.animation = Arc::clone(animations.as_ref().animations.get(animation).expect(format!("Failed to get animation {}", animation).as_str()));
    }

    pub fn advance_animation_state(&mut self)  {
        self.animation_state += 1;
        if self.animation_state >= self.animation.frames.len() {
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
        path: Vec<(usize, usize, usize)>,
        destination: (usize, usize, usize),
    },
    FindPath {
        destination: (usize, usize, usize),
    },
    Plough {
        target: (usize, usize, usize)
    },
    Dig {
        target: (usize, usize, usize),
    },
    Place {
        target: (usize, usize, usize),
        cube: Cube
    }
}


impl AgentTask {

    /**
    * return the positions required by the agent in order to perform this task
    */
    pub fn get_required_position(&self) -> Option<Vec<(usize, usize, usize)>> {
        match self {
            AgentTask::Move { .. } => None,
            AgentTask::FindPath { .. } => None,
            AgentTask::Plough { target } => Some(vec![(target.0, target.1, target.2 + 1)]),
            AgentTask::Dig { target } => Some(find_horizontal_neighbours((target.0, target.1, target.2 + 1))),
            AgentTask::Place { target, .. } => Some(find_horizontal_neighbours((target.0, target.1, target.2 + 1)))
        }
    }
}