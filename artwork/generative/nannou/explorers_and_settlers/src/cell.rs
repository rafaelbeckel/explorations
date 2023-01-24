use std::cell::RefCell;
use std::collections::HashMap;

use nannou::prelude::*;

use crate::agent::Agent;

pub enum CellState {
    Empty,
    Filled {
        by: RefCell<Agent>,
        times: i32,
        blocked: bool,
    },
}

pub struct Cell {
    pub row: i32,
    pub col: i32,
    pub rect: Rect,
    pub state: CellState,
}

impl Cell {
    pub fn new(row: i32, col: i32, rect: Rect) -> Self {
        Cell {
            row,
            col,
            rect,
            state: CellState::Empty,
        }
    }

    fn get_neighbors(&self, cells_map: &HashMap<String, usize>) -> Vec<usize> {
        let mut neighbors: Vec<usize> = Vec::new();

        let mut row = self.row;
        let mut col = self.col;

        // top
        row -= 1;
        if row >= 0 {
            let key = format!("{}{}", row, col);
            if let Some(index) = cells_map.get(&key) {
                neighbors.push(*index);
            }
        }

        // bottom
        row += 2;
        if row < 100 {
            let key = format!("{}{}", row, col);
            if let Some(index) = cells_map.get(&key) {
                neighbors.push(*index);
            }
        }

        // left
        row -= 1;
        col -= 1;
        if col >= 0 {
            let key = format!("{}{}", row, col);
            if let Some(index) = cells_map.get(&key) {
                neighbors.push(*index);
            }
        }

        // right
        col += 2;
        if col < 100 {
            let key = format!("{}{}", row, col);
            if let Some(index) = cells_map.get(&key) {
                neighbors.push(*index);
            }
        }

        neighbors
    }

    pub fn fill(&mut self, agent: RefCell<Agent>) {
        // set state to filled or increase N times if it's the same agent
        match self.state {
            CellState::Empty => {
                self.state = CellState::Filled {
                    by: agent,
                    times: 1,
                    blocked: false,
                }
            }
            CellState::Filled {
                by: _,
                times,
                blocked,
            } => {
                if !blocked {
                    self.state = CellState::Filled {
                        by: agent,
                        times: times + 1,
                        blocked: true,
                    }
                    // @TODO: check if the RefCell is the same as the one in the state
                    // } else if by == agent {
                    //     self.state = CellState::Filled {
                    //         by,
                    //         times: times + 1,
                    //         blocked: false,
                    //     }
                }
            }
        }
    }

    pub fn draw(&self, draw: &Draw, color: Hsv) {
        let x = self.rect.x();
        let y = self.rect.y();
        let w = self.rect.w();
        let h = self.rect.h();

        draw.rect().color(color).x_y(x, y).w_h(w, h);
    }
}
