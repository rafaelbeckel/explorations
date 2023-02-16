use nannou::color::*;
use nannou::prelude::*;

struct Particle {
    position: Point3,
    velocity: Vec3,
    color: Rgb,
    radius: f32,
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

    fn update(&mut self, dt: f32) {
        // change the force to attract to the center
        let center = Point3::new(0.0, 0.0, 0.0);
        let force = (center - self.position).normalize();

        self.velocity += force * dt * 100.0;
        self.position += self.velocity * dt;
        self.radius = 10.0 + self.position.z / 50.0;
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
}

struct Model {
    _window: WindowId,
    window_size: Vec2,
    particle_system: ParticleSystem,
}

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let window = app.new_window().view(view).event(event).build().unwrap();
    let window_size = app.window_rect().wh();

    let mut particle_system = ParticleSystem::new();
    let center = Point3::new(0.0, 0.0, 0.0);
    let num_particles = 2000;
    for i in 0..num_particles {
        let i = i as f32;
        let num_particles = num_particles as f32;
        let angle = i / num_particles * TAU;
        let pos = center + Point3::new(angle.cos(), angle.sin(), angle) * i;
        let vel = Vec3::new(i / 10.0 as f32, -i / 10.0 as f32, angle);
        let color = hsl(i / num_particles, 0.8, 0.5).into();
        let radius = 1.0 + i / num_particles * 10.0;

        let particle = Particle::new(pos, vel, color, radius);
        particle_system.add_particle(particle);
    }

    Model {
        _window: window,
        window_size,
        particle_system,
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

fn update(_app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();
    model.particle_system.update(dt);
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
