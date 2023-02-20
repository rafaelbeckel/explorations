// Credits: https://github.com/BoyBaykiller/Newtonian-Particle-Simulator

#version 450
#define EPSILON 0.001
const float DRAG_COEF = log(0.998) * 176.0; // log(0.70303228048)

struct Particle
{
    vec3 Position;
    vec3 Velocity;
};

layout(std430, binding = 0) restrict buffer ParticlesSSBO
{
    Particle Particles[];
} particleSSBO;

layout(location = 0) out vec4 out_color;

void main()
{
    Particle particle = particleSSBO.Particles[gl_VertexIndex];
    
    /// Implementation of Newton's law of gravity
    const float G = 1.0; // gravitational constant 
    const float m1_m2 = 176.0; // mass of both objects multiplied
    
    // acceleration = toMass * force. Technically toMass would have to be normalized but feels better without

    particleSSBO.Particles[gl_VertexIndex] = particle;

    const float red = 0.0045 * dot(particle.Velocity, particle.Velocity);
    const float green = clamp(0.08 * max(particle.Velocity.x, max(particle.Velocity.y, particle.Velocity.z)), 0.2, 0.5);
    const float blue = 0.7 - red;

    out_color = vec4(red, green, blue, 1.0);
    gl_Position = vec4(particle.Position, 1.0);
}