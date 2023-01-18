use std::collections::HashMap;

use nannou::color::*;
use nannou::prelude::*;

struct Agent {
    settle: bool,
}

enum CellState {
    Empty,
    Filled { by: Agent, times: f32 },
}

struct Cell {
    row: i32,
    col: i32,
    rect: Rect,
    state: CellState,
}

struct Model {
    _window: WindowId,
    window_size: Vec2,
    cell_size: f32,
    warm_palette: Vec<Hsv>,
    cool_palette: Vec<Hsv>,
    muted_warm_palette: Vec<Hsv>,
    muted_cool_palette: Vec<Hsv>,
    cells: Vec<Cell>,
    cells_map: HashMap<String, usize>,
    cell_spacing: f32,
    animation_phase: f32,
    epoch: usize,
    agents: Vec<Agent>,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app.new_window().view(view).event(event).build().unwrap();
    let window_size = app.window_rect().wh();
    let cell_size = 16.0; // Set this to the size of each square.
    let cell_spacing = 2.0; // Set this to the space between each square.

    let num_colors: i32 = 360;
    let warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.6, 0.8, 0.6, 0.8);
    let cool_palette: Vec<Hsv> = create_pallete(num_colors, 181.0, 360.0, 0.6, 0.8, 0.6, 0.8);
    let muted_warm_palette: Vec<Hsv> = create_pallete(num_colors, 1.0, 180.0, 0.0, 0.01, 0.6, 0.8);
    let muted_cool_palette: Vec<Hsv> =
        create_pallete(num_colors, 181.0, 360.0, 0.0, 0.01, 0.6, 0.8);

    let n_cols = (window_size.x / cell_size) as i32;
    let n_rows = (window_size.y / cell_size) as i32;
    let mut cells_map = HashMap::new();
    let mut index: usize = 0;

    let cells: Vec<Cell> = (0..n_cols)
        .flat_map(|col| {
            (0..n_rows).map(move |row| {
                let x = col as f32 * (cell_size + cell_spacing) - (window_size.x / 2.0);
                let y = row as f32 * (cell_size + cell_spacing) - (window_size.y / 2.0);
                let rect = Rect::from_xy_wh(Vec2::new(x, y), Vec2::new(cell_size, cell_size));

                Cell {
                    row,
                    col,
                    rect,
                    state: CellState::Empty,
                }
            })
        })
        .collect();

    for cell in cells.iter() {
        cells_map.insert(format!("{}{}", cell.row, cell.col), index);
        index += 1;
    }

    Model {
        _window: window,
        window_size,
        cell_size,
        warm_palette,
        cool_palette,
        muted_warm_palette,
        muted_cool_palette,
        cells,
        cells_map,
        cell_spacing,
        animation_phase: 0.0,
        epoch: 0,
        agents: vec![Agent { settle: false }],
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

    let mut cells_map = HashMap::new();
    let mut index: usize = 0;

    let cells: Vec<Cell> = (0..n_cols)
        .flat_map(|col| {
            (0..n_rows).map(move |row| {
                let x = col as f32 * (cell_size + cell_spacing) - (window_size.x / 2.0);
                let y = row as f32 * (cell_size + cell_spacing) - (window_size.y / 2.0);
                let rect = Rect::from_xy_wh(Vec2::new(x, y), Vec2::new(cell_size, cell_size));

                Cell {
                    row,
                    col,
                    rect,
                    state: CellState::Empty,
                }
            })
        })
        .collect();

    for cell in cells.iter() {
        cells_map.insert(format!("{}{}", cell.row, cell.col), index);
        index += 1;
    }

    model.cells = cells;
    model.cells_map = cells_map;
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

    for (i, cell) in model.cells.iter().enumerate() {
        let rect = cell.rect;
        let warm_color;
        let cool_color;

        match cell.state {
            CellState::Empty => {
                warm_color = model.muted_warm_palette[i % model.muted_warm_palette.len()];
                cool_color = model.muted_cool_palette[i % model.muted_cool_palette.len()];
            }
            CellState::Filled { by: _, times: _ } => {
                warm_color = model.warm_palette[i % model.warm_palette.len()];
                cool_color = model.cool_palette[i % model.cool_palette.len()];
            }
            _ => {
                warm_color = model.warm_palette[i % model.warm_palette.len()];
                cool_color = model.cool_palette[i % model.cool_palette.len()];
            }
        }

        let color = warm_color.mix(&cool_color, model.animation_phase);

        let x = rect.x();
        let y = rect.y();
        let w = rect.w();
        let h = rect.h();

        draw.rect()
            .color(Hsv::new(color.hue, color.saturation, color.value))
            .x_y(x, y)
            .w_h(w, h);
    }

    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}
