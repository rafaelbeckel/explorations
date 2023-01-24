use nannou::prelude::*;

pub struct Agent {
    pub position: Vec2,
    pub settle: bool,
}

impl Agent {
    pub fn new(position: Vec2) -> Self {
        Agent {
            position,
            settle: false,
        }
    }
}
