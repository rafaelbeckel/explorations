use nannou::prelude::*;

pub struct Agent {
    pub id: String,
    pub position: Vec2,
    pub settle: bool,
}

impl Agent {
    pub fn new(position: Vec2) -> Self {
        Agent {
            id: format!("{}{}", position.x, position.y),
            position,
            settle: false,
        }
    }
}
