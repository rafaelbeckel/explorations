use std::cell::RefCell;
use std::collections::HashMap;

use nannou::color::*;
use nannou::prelude::*;

struct Agent {
    position: Vec2,
    settle: bool,
}

impl Agent {
    fn new(position: Vec2) -> Self {
        Agent {
            position,
            settle: false,
        }
    }
}

struct Grid {
    n_cols: i32,
    n_rows: i32,
    cell_size: f32,
    cell_spacing: f32,
    cells: Vec<Cell>,
    cells_map: HashMap<String, usize>,
}

impl Grid {
    fn new(n_cols: i32, n_rows: i32, cell_size: f32, cell_spacing: f32) -> Self {
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

    fn fill(&mut self, index: usize, agent: RefCell<Agent>) {
        self.cells[index].fill(agent);
    }
}

enum CellState {
    Empty,
    Filled {
        by: RefCell<Agent>,
        times: i32,
        blocked: bool,
    },
}

struct Cell {
    row: i32,
    col: i32,
    rect: Rect,
    state: CellState,
}

impl Cell {
    fn new(row: i32, col: i32, rect: Rect) -> Self {
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

    fn fill(&mut self, agent: RefCell<Agent>) {
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

    fn draw(&self, draw: &Draw, color: Hsv) {
        let x = self.rect.x();
        let y = self.rect.y();
        let w = self.rect.w();
        let h = self.rect.h();

        draw.rect().color(color).x_y(x, y).w_h(w, h);
    }
}

struct Model {
    _window: WindowId,
    window_size: Vec2,
    cell_size: f32,
    warm_palette: Vec<Hsv>,
    cool_palette: Vec<Hsv>,
    muted_warm_palette: Vec<Hsv>,
    muted_cool_palette: Vec<Hsv>,
    grid: Grid,
    cell_spacing: f32,
    animation_phase: f32,
    epoch: usize,
    agents: Vec<Agent>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    // Window
    let window = app.new_window().view(view).event(event).build().unwrap();
    let window_size = app.window_rect().wh();

    // Grid
    let cell_size = 16.0; // Set this to the size of each square.
    let cell_spacing = 2.0; // Set this to the space between each square.
    let n_cols = (window_size.x / cell_size) as i32;
    let n_rows = (window_size.y / cell_size) as i32;
    let grid = Grid::new(n_cols, n_rows, cell_size, cell_spacing);

    // Color Palettes
    let num_colors: i32 = 360;
    let warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.6, 0.9, 0.6, 0.8);
    let cool_palette: Vec<Hsv> = create_pallete(num_colors, 181.0, 360.0, 0.6, 0.9, 0.6, 0.8);
    let muted_warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.0, 0.1, 0.6, 0.8);
    let muted_cool_palette: Vec<Hsv> = create_pallete(num_colors, 181.0, 360.0, 0.0, 0.1, 0.6, 0.8);

    // max agents is the number of cells in the grid divided by 10
    let max_agents = (n_cols * n_rows) / 10;

    //create the agents in random places, but not near each other
    let agents: Vec<Agent> = (0..max_agents)
        .map(|_| {
            let mut agent = Agent::new(Vec2::new(
                random_range(0, n_cols) as f32,
                random_range(0, n_rows) as f32,
            ));
            agent.settle = random::<bool>();
            agent
        })
        .collect();

    Model {
        _window: window,
        window_size,
        cell_size,
        warm_palette,
        cool_palette,
        muted_warm_palette,
        muted_cool_palette,
        grid,
        cell_spacing,
        animation_phase: 0.0,
        epoch: 0,
        agents,
    }
}

fn create_pallete(
    num_colors: i32,
    min_hue: f32,
    max_hue: f32,
    min_saturation: f32,
    max_saturation: f32,
    min_value: f32,
    max_value: f32,
) -> Vec<Hsv> {
    let mut palette: Vec<Hsv> = (0..num_colors)
        .map(|i| {
            Hsv::new(
                map_range(i, 0, num_colors - 1, min_hue, max_hue),
                map_range(i, 0, num_colors - 1, min_saturation, max_saturation),
                map_range(i, 0, num_colors - 1, min_value, max_value),
            )
        })
        .collect();

    let inverted_palette: Vec<Hsv> = palette.clone().into_iter().rev().collect();
    palette.extend(inverted_palette);

    palette
}

fn update_model(model: &mut Model) {
    let window_size = model.window_size;
    let cell_size = model.cell_size; // Set this to the size of each square in pixels.
    let cell_spacing = model.cell_spacing;
    let n_cols = (window_size.x / cell_size) as i32;
    let n_rows = (window_size.y / cell_size) as i32;

    model.grid = Grid::new(n_cols, n_rows, cell_size, cell_spacing);

    let max_agents = (n_cols * n_rows) / 10;
    let agents: Vec<Agent> = (0..max_agents)
        .map(|_| {
            let mut agent = Agent::new(Vec2::new(
                random_range(0, n_cols) as f32,
                random_range(0, n_rows) as f32,
            ));
            agent.settle = random::<bool>();
            agent
        })
        .collect();

    model.agents = agents;
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        Resized(size) => {
            model.window_size = size;
            update_model(model);
        }
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.animation_phase = (app.time).sin() / 2.0 + 0.5;

    if app.elapsed_frames() % 10 == 0 {
        model.epoch += 1;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    for (i, cell) in model.grid.cells.iter().enumerate() {
        let mut warm_color;
        let cool_color;

        match cell.state {
            CellState::Empty => {
                warm_color = model.muted_warm_palette[i % model.muted_warm_palette.len()];
                cool_color = model.muted_cool_palette[i % model.muted_cool_palette.len()];
            }
            CellState::Filled {
                by: _,
                times,
                blocked: _,
            } => {
                warm_color = model.warm_palette[i % model.warm_palette.len()];
                cool_color = model.cool_palette[i % model.cool_palette.len()];

                // change intensity based on how many times it's been filled
                warm_color.saturation = map_range(times, 0, 5, 0.5, 1.0);
            }
        }

        let color = warm_color.mix(&cool_color, model.animation_phase);

        cell.draw(&draw, color);
    }

    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}
