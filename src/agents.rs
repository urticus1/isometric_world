use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use crate::{Compass, Sprite};
use crate::agents::AgentEvent::AgentAddTask;
use crate::agents::Direction::{Nx, Ny, Px, Py};
use crate::animation::{Animation, AnimationPool};
use crate::cubes::Cube;
use crate::events::GridChangeEvent;
use crate::grid::{Grid, WorldPos};

pub fn find_path(start: WorldPos, end: WorldPos, grid: &Grid) -> Option<Vec<WorldPos>> {
    let mut g_scores: HashMap<WorldPos, f32> = HashMap::new();
    let mut came_from: HashMap<WorldPos, WorldPos> = HashMap::new();
    g_scores.insert(start, 0.0);

    let start = Node {
        f: distance(start, end),
        g: 0.0,
        position: start
    };
    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    open.push(start);

    while !open.is_empty() {
        let current = open.pop().unwrap();

        if current.position == end {
            return Some(reconstruct_path(current.position, &came_from));
        }

        for neighbour in get_walkable_neighbours(current.position, grid) {
            let tentative_g = current.g + 1.0;
            if let Some(visited) = g_scores.get(&neighbour) {
                if *visited <= tentative_g {
                    continue;
                }
            }
            let node = Node {
                f: tentative_g + distance(neighbour, end),
                g: tentative_g,
                position: neighbour,
            };
            open.push(node);
            g_scores.insert(neighbour, tentative_g);
            came_from.insert(neighbour, current.position);
        }
    }
    println!("no path found");
    None

}

/**
*   return reversed path
*/
fn reconstruct_path(end: WorldPos, parents: &HashMap<WorldPos, WorldPos>) -> Vec<WorldPos> {
    let mut path = Vec::new();
    let mut current = end;
    path.push(current.clone());
    while let Some(node) = parents.get(&current) {
        path.push(node.clone());
        current = *node;
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
    f: f32,
    g: f32,
    position: WorldPos
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.total_cmp(&self.f)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g && self.position == other.position
    }
}

impl Eq for Node {}

fn get_walkable_neighbours(position: WorldPos, grid: &Grid) -> Vec<WorldPos> {
    [position.next_x(), position.prev_x(), position.next_y(), position.prev_y()]
        .into_iter()
        .flatten()
        .map(|pos| (pos, grid.get_cube(pos).unwrap()))
        .filter(|(_, cube)| cube.agent.is_none())
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

pub fn advance_task(agent: &mut Agent, grid: &mut Grid) {
    let mut completed = false;
    if let Some(task_wrapper) = &mut agent.active_task {
        match &mut task_wrapper.task {
            AgentTask::Move { path, ..} => {
                if let Some(next) = path.pop() {
                    if !grid.is_occupied(next) {
                        let x_dir = next.x as i32 - agent.position.x as i32;
                        let y_dir = next.y as i32 - agent.position.y as i32;
                        if x_dir < 0 {
                            agent.direction = Nx;
                        }
                        else if x_dir > 0 {
                            agent.direction = Px;
                        }
                        else if y_dir < 0 {
                            agent.direction = Ny;
                        }
                        else {
                            agent.direction = Py;
                        }
                        grid.move_agent(agent.position, next);
                        agent.position = next;

                    }
                    else {
                        if let Some(destination) = agent.destination {
                            if let Some(path) = find_path(agent.position, destination, grid) {
                                agent.tasks.insert(0, AgentTask::Move {
                                    path: path,
                                    destination: destination,
                                });
                            }
                        }
                        completed = true;
                    }
                }
                else {
                    completed = true;
                }
            }
            AgentTask::FindPath { destination } => {
                let start = agent.destination.or(Some(agent.position)).unwrap();
                {
                    if let Some(path) = find_path(start, *destination, grid) {
                        agent.tasks.insert(0, AgentTask::Move {
                            path: path,
                            destination: destination.clone(),
                        });
                    }

                    completed = true;
                }

            }
            AgentTask::Plough { .. } => {


                completed = true;
            }
            AgentTask::Dig { target } => {
                if agent.position.z > 0 {
                    grid.delete_cube(*target);
                }
                completed = true;
            }
            AgentTask::Place { target, cube } => {
                grid.place_cube(*target, cube.clone());
                completed = true;
            }
        }
    }
    if completed {
        if let Some(task_wrapper) = &mut agent.active_task {
            task_wrapper.completed = true;
        }
    }
}

pub fn run_agent_loop(game_tick: usize, grid: &mut Grid, agents: &mut Vec<Agent>, game_events_receiver: &Receiver<AgentEvent>, grid_events_receiver: &Receiver<GridChangeEvent>) {

    for event in grid_events_receiver.try_iter() {
        grid.handle_grid_change_event(event);
    }
    {
        match game_events_receiver.try_recv() {
            Ok(ev) => {
                match ev {
                    AgentAddTask { agent, task } => {
                        println!("got task");
                        agents[agent].tasks.push(task);
                    },
                }

            },
            Err(_) => {},
        }
        for agent in agents.iter_mut() {
            agent.advance_animation_state();
            if let Some(active_task) = &agent.active_task {
                if game_tick < active_task.end_tick {
                    continue;
                }
                advance_task(agent, grid);

                if let Some(task) = &agent.active_task {
                    if !task.completed {
                        continue;
                    }
                    agent.active_task = None;
                }
            }

            if agent.tasks.is_empty() {
                agent.change_animation("idle");
                continue;
            }
            let mut next_task = agent.tasks.remove(0);
            let mut cancel_task = false;
            if let Some(required_positions) = next_task.get_required_position() {
                if !required_positions.contains(&agent.position) {
                    println!("not in required pos");
                    for pos in required_positions {
                        {
                            if let Some(path) = find_path(agent.position, pos, grid) {
                                agent.tasks.insert(0, next_task.clone());
                                next_task = AgentTask::Move {
                                    destination: pos,
                                    path: path,
                                };
                                cancel_task = false;
                                break;
                            }
                        }
                        cancel_task = true;
                    }
                }
            }

            let end_tick = match &next_task {
                AgentTask::Plough {..} => {
                    agent.change_animation("ploughing");
                    game_tick + 15
                },

                AgentTask::Move { .. } => {
                    agent.change_animation("running");
                    game_tick + 1
                }
                AgentTask::FindPath { .. } => {
                    game_tick
                }
                AgentTask::Dig { .. } => {
                    agent.change_animation("mining");
                    game_tick + 20
                }
                AgentTask::Place { .. } => {
                    game_tick + 9
                }
            };
            let task = AgentCoroutine {
                task: next_task,
                end_tick,
                completed: false,
            };
            agent.active_task = if cancel_task { None } else { Some(task) };

        }
    }
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
    pub end_tick: usize,
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