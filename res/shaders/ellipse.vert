
#version 460 core

layout (location = 0) in vec2 a_top_left;
layout (location = 1) in vec2 a_top_right;
layout (location = 2) in vec2 a_bottom_right;
layout (location = 3) in vec4 a_color;
layout (location = 4) in float a_depth;

out vec4 v_color;
out vec2 v_coord;

uniform mat4 u_vp;

const vec2 coords[4] = vec2[4](
    vec2( 1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2(-1.0, -1.0)
);

void main() {
    vec2 bottom_left = a_top_left + a_bottom_right - a_top_right;
    vec2 positions[4] = vec2[](
        a_top_right,
        a_bottom_right,
        a_top_left,
        bottom_left
    );

    vec4 proj_pos = u_vp * vec4(positions[gl_VertexID], 0, 1);
    gl_Position = vec4(proj_pos.xy, a_depth, 1.0);

    v_color = a_color;
    v_coord = coords[gl_VertexID];
}
