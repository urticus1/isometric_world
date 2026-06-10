use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use crate::{Sprite, CUBE_TYPE_MASK, EMPTY_CUBE, GRID_WIDTH};
use crate::grid::Grid;

pub fn find_path(start: (usize, usize, usize), end: (usize, usize, usize), grid: &Grid) -> Vec<(usize, usize, usize)> {
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
                return reconstruct_path(Rc::clone(&nodes[&end]));
            }
        }
        closed.insert(current.position, current);

    }

    vec![]
}

fn reconstruct_path(end: Rc<Node>) -> Vec<(usize, usize, usize)> {
    let mut path = Vec::new();
    let mut current = Some(end);

    while let Some(node) = current {
        path.push(node.position);
        current = node.parent.clone();
    }

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
    if position.0 > 0 && grid[grid.get_vector_pos((position.0 - 1, position.1, position.2))].is_walkable() {
        neighbours.push((position.0 - 1, position.1, position.2));
    }
    if position.0 < GRID_WIDTH - 1 && grid[grid.get_vector_pos((position.0 + 1, position.1, position.2))].is_walkable() {
        neighbours.push((position.0 + 1, position.1, position.2));
    }
    if position.1 > 0 && grid[grid.get_vector_pos((position.0, position.1 - 1, position.2))].is_walkable() {
        neighbours.push((position.0, position.1 - 1, position.2));
    }
    if position.1 < GRID_WIDTH - 1 && grid[grid.get_vector_pos((position.0 , position.1 + 1 , position.2))].is_walkable() {
        neighbours.push((position.0, position.1 + 1, position.2));
    }
    neighbours
}

pub struct Agent {
    pub name: String,
    pub animation:  Arc<Animation>,
    pub animation_state: usize,
    pub position: (usize, usize, usize),
    pub task: Option<AgentTask>
}

pub enum AgentTask {
    Move(Vec<(usize, usize, usize)>),
}

pub struct Animation {
    pub frames: Vec<Sprite>,
    pub name: String
}