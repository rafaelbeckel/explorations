use std::collections::HashMap;

use nannou::prelude::*;

use crate::agent::Agent;

pub enum CellState {
    Empty,
    Filled {
        by: String,
        times: i32,
        blocked: bool,
    },
}

pub struct Cell {
    pub row: usize,
    pub col: usize,
    pub rect: Rect,
    pub state: CellState,
}

impl Cell {
    pub fn new(row: usize, col: usize, rect: Rect) -> Self {
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

    pub fn fill(&mut self, agent: &Agent) {
        self.state = match &self.state {
            CellState::Empty => CellState::Filled {
                by: agent.id.clone(),
                times: 1,
                blocked: false,
            },
            CellState::Filled { by, times, blocked } => {
                let same_agent = by.eq(&agent.id);
                if !blocked && same_agent && times.clone() < 6 {
                    CellState::Filled {
                        by: agent.id.clone(),
                        times: times + 1,
                        blocked: false,
                    }
                } else if same_agent {
                    CellState::Filled {
                        by: agent.id.clone(),
                        times: times.clone(),
                        blocked: true,
                    }
                } else {
                    CellState::Filled {
                        by: by.clone(),
                        times: times.clone(),
                        blocked: blocked.clone(),
                    }
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
