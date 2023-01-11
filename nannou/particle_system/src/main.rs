use nannou::color::*;
use nannou::prelude::*;

// struct Cylinder {
//     length: f32,
//     radius: f32,
//     volume: f32,
//     rotational_velocity: f32,
// }

// struct Fluid {
//     particles: Vec<Particle>,
//     particles_count: usize,
//     t0_particle_count: usize,
//     steady_particle_count: usize,
//     average_particle_concentration: f32,
//     mean_interparticle_separation: f32,
//     gravity: f32,
//     flow_based_reynolds_number: f32,
//     particle_based_reynolds_number: f32,
//     viscosity: f32,
//     density: f32,
//     kinematic_viscosity: f32,
// }

// To implement:
// G Green’s function for the Stokes equation

struct Particle {
    position: Point3,
    velocity: Vec3,
    color: Rgb,
    radius: f32,
    // settling_velocity: Vec3,
    // floating_velocity: Vec3,
    // radial_velocity: Vec3,
    // angular_velocity: Vec3,
    // surface: f32,
    // volume: f32,
    // mass: f32,
    // buoyancy_corrected_mass: f32,
    // steady: bool,
    // density: f32,
}

struct ParticleSystem {
    particles: Vec<Particle>,
}

impl Particle {
    fn new(position: Point3, velocity: Vec3, color: Rgb, radius: f32) -> Self {
        Particle {
            position,
            velocity,
            color,
            radius,
        }
    }

    fn update(&mut self, dt: f32, force: Vec3) {
        self.velocity += force * dt;
        self.position += self.velocity * dt;
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

    fn update(&mut self, dt: f32, force: Vec3) {
        for particle in self.particles.iter_mut() {
            particle.update(dt, force);
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
    particle_system: ParticleSystem,
    time: f32,
    dt: f32,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app.new_window().view(view).event(event).build().unwrap();
    let window_size = app.window_rect().wh();

    let mut particle_system = ParticleSystem::new();
    let center = Point3::new(0.0, 0.0, 0.0);
    let num_particles = 500;
    for i in 0..num_particles {
        let i = i as f32;
        let num_particles = num_particles as f32;
        let angle = i / num_particles * TAU;
        let pos = center + Point3::new(angle.cos(), angle.sin(), 0.0) * 100.0;
        let vel = Vec3::new(0.0, 0.0, 0.0);
        let color = hsl(i / num_particles, 0.8, 0.5).into();
        let radius = 1.0 + i / num_particles * 10.0;

        let particle = Particle::new(pos, vel, color, radius);
        particle_system.add_particle(particle);
    }

    Model {
        _window: window,
        window_size,
        particle_system,
        time: 0.0,
        dt: 0.0,
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        Resized(size) => {
            model.window_size = size;
        }
        _ => (),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();
    let time = update.since_start.as_secs_f32();

    model.time = time * 2.0;
    model.dt = dt;

    let force = Vec3::new(time.cos() * 20.0, 0.0, 0.0) * 2.0;

    model.particle_system.update(dt, force);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    for particle in &model.particle_system.particles {
        draw.ellipse()
            .color(particle.color)
            .radius(particle.radius)
            .x_y(particle.position.x, particle.position.y);
    }

    draw.background().color(BLACK);
    draw.to_frame(app, &frame).unwrap();
}
