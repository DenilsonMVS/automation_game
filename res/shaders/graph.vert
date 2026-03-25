#version 460 core

uniform mat4 u_projection;
uniform float u_display_proportion;

layout(std430, binding = 0) buffer GraphData {
    vec2 center;
    vec2 dimensions;
    vec2 view_min;
    vec2 view_max;
    uint primary_color;
    float thickness;
    float bellow_alpha;
    float y_values[];
};

out vec2 v_coord;

void main() {
    const vec2 mults[] = vec2[](
        vec2(                   0, 0),
        vec2(u_display_proportion, 0),
        vec2(u_display_proportion, 1),
        vec2(                   0, 1)
    );

    vec2 bottom_left = center - dimensions * 0.5;
    vec2 pos = bottom_left + dimensions * mults[gl_VertexID % 4];

    vec4 t_pos = u_projection * vec4(pos, 0, 1);
    gl_Position = vec4(t_pos.xy, 0.99, 1);
    v_coord = mults[gl_VertexID % 4];
}