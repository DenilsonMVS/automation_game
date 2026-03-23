
#version 460 core

uniform vec2 u_center;
uniform vec2 u_dimensions;
uniform mat4 u_projection;

out vec2 v_coord;

void main() {
    const vec2 mults[] = vec2[](
        vec2(0, 0),
        vec2(1, 0),
        vec2(1, 1),
        vec2(0, 1)
    );

    const vec2 bottom_left = u_center - u_dimensions * 0.5;
    const vec2 vertices[] = vec2[](
        bottom_left + u_dimensions * mults[0],
        bottom_left + u_dimensions * mults[1],
        bottom_left + u_dimensions * mults[2],
        bottom_left + u_dimensions * mults[3]
    );

    gl_Position = u_projection * vec4(vertices[gl_VertexID % 4], 0, 1);
    v_coord = mults[gl_VertexID % 4];
}
