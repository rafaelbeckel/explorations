use cell::CellState;
use nannou::color::*;
use nannou::prelude::*;

mod agent;
use crate::agent::Agent;

mod grid;
use crate::grid::Grid;

mod cell;

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
    let n_cols = (window_size.x / cell_size) as usize;
    let n_rows = (window_size.y / cell_size) as usize;
    let mut grid = Grid::new(n_cols, n_rows, cell_size, cell_spacing);

    // Color Palettes
    let num_colors: i32 = 360;
    let warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.6, 0.9, 0.6, 0.8);
    let cool_palette: Vec<Hsv> = create_pallete(num_colors, 181.0, 360.0, 0.6, 0.9, 0.6, 0.8);
    let muted_warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.0, 0.1, 0.6, 0.8);
    let muted_cool_palette: Vec<Hsv> = create_pallete(num_colors, 181.0, 360.0, 0.0, 0.1, 0.6, 0.8);

    // max agents is the number of cells in the grid divided by 10
    let max_agents = (n_cols * n_rows) / 10;

    // create the agents in random places
    let agents: Vec<Agent> = (0..max_agents)
        .map(|_| {
            let row = random_range(0, n_rows);
            let col = random_range(0, n_cols);
            let mut agent = Agent::new(Vec2::new(row as f32, col as f32));
            agent.settle = random::<bool>();

            grid.fill(row, col, &agent);

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
    let n_cols = (window_size.x / cell_size) as usize;
    let n_rows = (window_size.y / cell_size) as usize;

    model.grid = Grid::new(n_cols, n_rows, cell_size, cell_spacing);

    // max agents is the number of cells in the grid divided by 10
    let max_agents = (n_cols * n_rows) / 10;

    // create the agents in random places
    let agents: Vec<Agent> = (0..max_agents)
        .map(|_| {
            let row = random_range(0, n_rows);
            let col = random_range(0, n_cols);
            let mut agent = Agent::new(Vec2::new(row as f32, col as f32));
            agent.settle = random::<bool>();

            model.grid.fill(row, col, &agent);

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

        // @TODO update the agents
        
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
