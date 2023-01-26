use nannou::prelude::*;

enum Direction {
    Settle,
    Left,
    Right,
    Up,
    Down,
}

pub struct Agent {
    pub id: String,
    pub position: Vec2,
    settle: bool,
    direction: Direction,
    intensity: i32, // how many epochs the agent will remain in the same state
}

impl Agent {
    pub fn new(position: Vec2) -> Self {
        let (settle, direction, intensity) = Self::explore_or_settle();

        Agent {
            id: format!("{}{}", position.x, position.y),
            position,
            settle,
            direction,
            intensity,
        }
    }

    // run when intensity reaches zero
    fn explore_or_settle() -> (bool, Direction, i32) {
        let settle = random::<bool>();

        let direction = if settle {
            Direction::Settle
        } else {
            let direction = random::<f32>();

            if direction < 0.25 {
                Direction::Left
            } else if direction < 0.5 {
                Direction::Right
            } else if direction < 0.75 {
                Direction::Up
            } else {
                Direction::Down
            }
        };

        let intensity = random_range(1, 10);

        (settle, direction, intensity)
    }

    // run every epoch
    // @TODO find a clean way to inject the cells map here
    // @TODO decide: should agents spawn children agents or paint adjacent cells?
    pub fn update() {
        // if direction is settle, expand to neighboring cells
        // if direction is not settle, move in that direction
        // if intensity reaches zero, run explore_or_settle
    }

    // run when agent is settled
    pub fn settle() {
        // paint every neighboring cell
        // if any neighboring cell is already settled, stop expanding
    }

    // run when agent is not settled
    pub fn explore() {
        // move in the direction
    }
}
