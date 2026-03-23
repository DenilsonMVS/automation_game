
#version 460 core

layout (location = 0) in vec2 a_v1;
layout (location = 1) in vec2 a_v2;
layout (location = 2) in vec2 a_v3;
layout (location = 3) in vec4 a_color;
layout (location = 4) in float a_depth;

out vec4 v_color;

uniform mat4 u_vp;

void main() {
    vec2 positions[3] = vec2[](
        a_v1,
        a_v2,
        a_v3
    );

    vec4 proj_pos = u_vp * vec4(positions[gl_VertexID % 3], 0, 1);
    gl_Position = vec4(proj_pos.xy, a_depth, 1.0);
    v_color = a_color;
}
