#version 460 core

layout (location = 0) in vec2 a_top_left; 
layout (location = 1) in vec2 a_top_right;
layout (location = 2) in vec4 a_color;
layout (location = 3) in float a_depth;
layout (location = 4) in uint a_glyph_index;

struct UvInfo {
    vec2 min;
    vec2 max;
};

layout(std430, binding = 5) buffer Uv {
    UvInfo info[];
};

out vec2 v_tex_coord;
out vec4 v_color;

uniform sampler2D u_texture;
uniform mat4 u_projection;

const vec2 OFFSETS[4] = vec2[](
    vec2(0.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0)
);

void main() {
    vec2 uv_min = info[a_glyph_index].min;
    vec2 uv_max = info[a_glyph_index].max;
    vec2 uv_size = uv_max - uv_min;
    vec2 atlas_size = vec2(textureSize(u_texture, 0));
    vec2 glyph_px_atlas_size = uv_size * atlas_size;

    float real_width = length(a_top_right - a_top_left);
    float real_height = real_width / glyph_px_atlas_size.x * glyph_px_atlas_size.y;
    vec2 next_dir = (a_top_right - a_top_left) / real_width;
    
    vec2 bellow_dir = vec2(next_dir.y, -next_dir.x);

    vec2 bl = a_top_left + bellow_dir * real_height;
    vec2 br = a_top_right + bellow_dir * real_height;
    
    vec2 positions[4] = vec2[](a_top_left, bl, a_top_right, br);

    vec4 transformed = u_projection * vec4(positions[gl_VertexID], 0, 1);
	gl_Position = vec4(transformed.xy, a_depth, 1.0);

    v_tex_coord = mix(uv_min, uv_max, OFFSETS[gl_VertexID]);
    v_color = a_color;
}
