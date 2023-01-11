use nannou::color::*;
use nannou::prelude::*;

struct Cylinder {
    length: f32,
    radius: f32,
    volume: f32,
    rotational_velocity: f32,
}

struct Fluid {
    particles: Vec<Particle>,
    particles_count: usize,
    t0_particle_count: usize,
    steady_particle_count: usize,
    average_particle_concentration: f32,
    mean_interparticle_separation: f32,
    gravity: f32,
    flow_based_reynolds_number: f32,
    particle_based_reynolds_number: f32,
    viscosity: f32,
    density: f32,
    kinematic_viscosity: f32,
}

// To implement:
// G Green’s function for the Stokes equation

struct Particle {
    pos: Point3,
    settling_velocity: Vec3,
    floating_velocity: Vec3,
    radial_velocity: Vec3,
    angular_velocity: Vec3,
    surface: f32,
    volume: f32,
    color: Rgb,
    radius: f32,
    mass: f32,
    buoyancy_corrected_mass: f32,
    steady: bool,
    density: f32,
}

impl Particle {
    fn new(pos: Point3, vel: Vec3, color: Rgb, radius: f32) -> Self {
        Particle {
            pos,
            vel,
            color,
            radius,
        }
    }

    fn update(&mut self, dt: f32, force: Vec3) {
        self.vel += force * dt;
        self.pos += self.vel * dt;
    }

    fn destroy(&mut self) {
        self.color = rgb(0.0, 0.0, 0.0);
        self.radius = 0.0;
    }
}

impl ParticleSystem {
    fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        for particle in self.particles.iter_mut() {
            particle.update(dt);
        }
    }

    fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    fn remove_particle(&mut self, index: usize) {
        self.particles.remove(index);
    }
}

struct Model {
    _window: WindowId,
    window_size: Vec2,
    rectangle_size: f32,
    warm_palette: Vec<Hsv>,
    cool_palette: Vec<Hsv>,
    grid: Vec<Rect>,
    space: f32,
    animation_phase: f32,
    rotation_animation_phase: f32,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app.new_window().view(view).event(event).build().unwrap();
    let window_size = app.window_rect().wh();
    let rectangle_size = 16.0; // Set this to the size of each square.
    let space = 2.0; // Set this to the space between each square.

    let num_colors: i32 = 360;
    let mut warm_palette: Vec<Hsv> = (0..num_colors)
        .map(|i| {
            Hsv::new(
                map_range(i, 0, num_colors - 1, 1.0, 180.0),
                map_range(i, 0, num_colors - 1, 0.6, 0.8),
                map_range(i, 0, num_colors - 1, 0.6, 0.8),
            )
        })
        .collect();

    let inverted_warm_palette: Vec<Hsv> = warm_palette.clone().into_iter().rev().collect();
    warm_palette.extend(inverted_warm_palette);

    let mut cool_palette: Vec<Hsv> = (0..num_colors)
        .map(|i| {
            Hsv::new(
                map_range(i, 0, num_colors - 1, 181.0, 360.0),
                map_range(i, 0, num_colors - 1, 0.6, 0.8),
                map_range(i, 0, num_colors - 1, 0.6, 0.8),
            )
        })
        .collect();

    let inverted_cool_palette: Vec<Hsv> = cool_palette.clone().into_iter().rev().collect();
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
        _window: window,
        window_size,
        rectangle_size,
        warm_palette,
        cool_palette,
        grid,
        space,
        animation_phase: 0.0,
        rotation_animation_phase: 0.0,
    }
}

fn update_model(model: &mut Model) {
    let window_size = model.window_size;
    let rectangle_size = model.rectangle_size; // Set this to the size of each square in pixels.
    let space = model.space;
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

    model.grid = grid;
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

    model.rotation_animation_phase += app.duration.since_prev_update.as_secs_f32() * 0.5;
    if model.rotation_animation_phase > 1.0 {
        model.rotation_animation_phase -= 1.0;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    for (i, rect) in model.grid.iter().enumerate() {
        let warm_color = model.warm_palette[i % model.warm_palette.len()];
        let cool_color = model.cool_palette[i % model.cool_palette.len()];

        let color = warm_color.mix(&cool_color, model.animation_phase);
        let x = rect.x() * model.animation_phase * 2.0 + rect.x();
        let y = rect.y() * model.animation_phase * 2.0 + rect.y();
        let w = rect.w() * model.animation_phase * 4.0 + 16.0;
        let h = rect.h() * model.animation_phase * 4.0 + 16.0;

        draw.rect()
            .color(Hsv::new(
                color.hue,
                0.5 + model.animation_phase / 2.0,
                color.value,
            ))
            .x_y(x, y)
            .w_h(w, h)
            .rotate(model.rotation_animation_phase * TAU + i as f32 * 0.1);
    }

    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}
