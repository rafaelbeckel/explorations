use std::cell::RefCell;
use std::collections::HashMap;

use nannou::prelude::*;

use crate::agent::Agent;
use crate::cell::Cell;

pub struct Grid {
    pub n_cols: i32,
    pub n_rows: i32,
    pub cell_size: f32,
    pub cell_spacing: f32,
    pub cells: Vec<Cell>,
    pub cells_map: HashMap<String, usize>,
}

impl Grid {
    pub fn new(n_cols: i32, n_rows: i32, cell_size: f32, cell_spacing: f32) -> Self {
        let mut cells_map = HashMap::new();

        let cells: Vec<Cell> = (0..n_cols)
            .flat_map(|col| {
                (0..n_rows).map(move |row| {
                    let x =
                        col as f32 * (cell_size + cell_spacing) - (n_cols as f32 * cell_size / 2.0);
                    let y =
                        row as f32 * (cell_size + cell_spacing) - (n_rows as f32 * cell_size / 2.0);
                    let rect = Rect::from_xy_wh(Vec2::new(x, y), Vec2::new(cell_size, cell_size));

                    Cell::new(row, col, rect)
                })
            })
            .collect();

        for (index, cell) in cells.iter().enumerate() {
            let key = format!("{}{}", cell.row, cell.col);
            cells_map.insert(key, index);
        }

        Grid {
            n_cols,
            n_rows,
            cell_size,
            cell_spacing,
            cells,
            cells_map,
        }
    }

    pub fn fill(&mut self, index: usize, agent: RefCell<Agent>) {
        self.cells[index].fill(agent);
    }
}
