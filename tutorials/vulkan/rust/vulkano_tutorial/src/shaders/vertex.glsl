#version 450

layout(location = 0) in vec2 position;

layout(location = 0) out vec3 out_color;

void main() {
    const vec3 colors[3] = vec3[3](
        vec3(1.0f, 0.0f, 0.0f), //red
        vec3(0.0f, 1.0f, 0.0f), //green
        vec3(0.0f, 0.0f, 1.0f)  //blue
    );

    gl_Position = vec4(position, 0.0, 1.0);
    out_color = colors[gl_VertexIndex];
}
