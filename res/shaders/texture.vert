#version 460 core

layout (location = 0) in vec2 a_top_left;
layout (location = 1) in vec2 a_top_right;
layout (location = 2) in uvec2 a_uv_min;
layout (location = 3) in uvec2 a_uv_max;

out vec2 v_tex_coord;

uniform mat4 u_projection;
uniform sampler2D u_texture; 

void main() {
    vec2 uv_pixels = vec2(a_uv_max) - vec2(a_uv_min);
    float aspect_ratio = abs(uv_pixels.y / uv_pixels.x);
    vec2 top_edge = a_top_right - a_top_left;
    vec2 side_edge = vec2(top_edge.y, -top_edge.x) * aspect_ratio;
    vec2 positions[4] = vec2[](
        a_top_left,
        a_top_right,
        a_top_left + side_edge,
        a_top_right + side_edge
    );

    vec2 uvs[4] = vec2[](
        vec2(uvec2(a_uv_min.x, a_uv_min.y)),
        vec2(uvec2(a_uv_max.x, a_uv_min.y)),
        vec2(uvec2(a_uv_min.x, a_uv_max.y)),
        vec2(uvec2(a_uv_max.x, a_uv_max.y))
    );

    vec2 tex_size = vec2(textureSize(u_texture, 0));
    v_tex_coord = uvs[gl_VertexID] / tex_size;

    gl_Position = u_projection * vec4(positions[gl_VertexID], 0.0, 1.0);
}
