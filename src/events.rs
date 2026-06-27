use std::collections::VecDeque;

pub struct EventQueue {
    pub cube_events: VecDeque<GridChangeEvent>,
}

pub enum GridChangeEvent {
    Delete((usize, usize, usize)),
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            cube_events: VecDeque::new(),
        }
    }

    pub fn publish_event(&mut self, grid_change_event: GridChangeEvent) {
        self.cube_events.push_back(grid_change_event);
    }
}