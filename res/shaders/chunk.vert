
#version 460 core

layout (location = 0) in vec4 a_color; 
layout (location = 1) in float a_size;

uniform mat4 u_vp;
uniform int u_cunk_mag;
uniform vec2 u_chunk_start_pos;

out vec4 v_color;

const vec2 OFFSETS[4] = vec2[](
    vec2(-0.5,  0.5),
    vec2(-0.5, -0.5),
    vec2( 0.5, -0.5),
    vec2( 0.5,  0.5)
);

void main() {
    int chunk_size = 1 << u_cunk_mag;
    vec2 center_position = vec2(ivec2(gl_InstanceID & (chunk_size - 1), gl_InstanceID >> u_cunk_mag));
    vec2 vertice_rel_position = center_position + a_size * OFFSETS[gl_VertexID % 4];
    vec2 vertice_global_position = u_chunk_start_pos + vertice_rel_position;

    vec2 transformed = (u_vp * vec4(vertice_global_position, 0, 1)).xy;
    gl_Position = vec4(transformed, 0.999999, 1);

    v_color = a_color;
}
