use nannou::color::*;
use nannou::prelude::*;

struct Model {
    window_size: Vec2,
    rectangle_size: f32,
    warm_palette: Vec<Hsv>,
    cool_palette: Vec<Hsv>,
    grid: Vec<Rect>,
    margin: f32,
    space: f32,
    animation_phase: f32,
}

fn main() {
    nannou::app(model)
        .update(update)
        //.event(event) // @todo handle window resize
        .simple_window(view)
        .run();
}

fn model(app: &App) -> Model {
    let window_size = app.window_rect().wh();
    let rectangle_size = 16.0; // Set this to the size of each square in pixels.
    let margin = 16.0;
    let space = 2.0;

    let num_colors: i32 = 180;
    let mut warm_palette: Vec<Hsv> = (0..num_colors)
        .map(|i| {
            Hsv::new(
                map_range(i, 0, num_colors - 1, 1.0, 180.0),
                map_range(i, 0, num_colors - 1, 0.5, 0.5),
                map_range(i, 0, num_colors - 1, 0.8, 0.8),
            )
        })
        .collect();

    let inverted_warm_palette: Vec<Hsv> = warm_palette.clone().into_iter().rev().collect();

    warm_palette.extend(inverted_warm_palette);

    let mut cool_palette: Vec<Hsv> = (0..num_colors)
        .map(|i| {
            Hsv::new(
                map_range(i, 0, num_colors - 1, 181.0, 360.0),
                map_range(i, 0, num_colors - 1, 0.5, 0.5),
                map_range(i, 0, num_colors - 1, 0.8, 0.8),
            )
        })
        .collect();

    let inverted_cool_palette: Vec<Hsv> = warm_palette.clone().into_iter().rev().collect();

    cool_palette.extend(inverted_cool_palette);

    let n_cols = (window_size.x / rectangle_size) as i32;
    let n_rows = (window_size.y / rectangle_size) as i32;

    let grid = (0..n_cols)
        .flat_map(|col| {
            (0..n_rows).map(move |row| {
                let x = col as f32 * (rectangle_size + space) - (window_size.x / 2.0);
                let y = row as f32 * (rectangle_size + space) - (window_size.y / 2.0);
                Rect::from_xy_wh(Vec2::new(x, y), Vec2::new(rectangle_size, rectangle_size))
            })
        })
        .collect();

    Model {
        window_size,
        rectangle_size,
        warm_palette,
        cool_palette,
        grid,
        margin,
        space,
        animation_phase: 0.0,
    }
}

// fn event(_app: &App, model: &mut Model, event: WindowEvent) {
//     match event {
//         Resized(size) => {
//             model.window_size = size;
//         }
//         _ => (),
//     }
// }

fn update(app: &App, model: &mut Model, _update: Update) {
    model.animation_phase = (app.time).sin() / 2.0 + 0.5;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    for (i, rect) in model.grid.iter().enumerate() {
        let warm_color = model.warm_palette[i % model.warm_palette.len()];
        let cool_color = model.cool_palette[i % model.cool_palette.len()];

        let color = warm_color.mix(&cool_color, model.animation_phase);
        let x = rect.x();
        let y = rect.y();
        let w = rect.w();
        let h = rect.h();

        draw.rect()
            .color(Hsv::new(
                color.hue,
                0.5 + model.animation_phase / 2.0,
                color.value,
            ))
            .x_y(x, y)
            .w_h(w, h);
    }

    draw.to_frame(app, &frame).unwrap();
}
