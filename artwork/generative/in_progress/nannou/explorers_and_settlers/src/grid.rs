use nannou::prelude::*;

use crate::agent::Agent;
use crate::cell::Cell;

pub struct Grid {
    pub n_cols: usize,
    pub n_rows: usize,
    pub cell_size: f32,
    pub cell_spacing: f32,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(n_cols: usize, n_rows: usize, cell_size: f32, cell_spacing: f32) -> Self {
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

        Grid {
            n_cols,
            n_rows,
            cell_size,
            cell_spacing,
            cells,
        }
    }

    pub fn fill(&mut self, row: usize, column: usize, agent: &Agent) {
        let index = row * self.n_cols + column;

        self.cells[index].fill(agent);
    }
}
