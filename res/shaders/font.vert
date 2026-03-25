#version 460 core

layout (location = 0) in ivec2 a_top_left; 
layout (location = 1) in ivec2 a_top_right;
layout (location = 2) in ivec2 a_bottom_right;
layout (location = 3) in vec3 a_color;
layout (location = 4) in uint a_glyph_index;

out vec2 v_tex_coord;
out vec3 v_color;

uniform sampler2D u_texture;
uniform uint u_glyph_size;
uniform uint u_glyph_margin;
uniform float u_world_scale;
uniform mat4 u_projection;

const vec2 OFFSETS[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 tl = vec2(a_top_left) * u_world_scale;
    vec2 tr = vec2(a_top_right) * u_world_scale;
    vec2 br = vec2(a_bottom_right) * u_world_scale;
    vec2 bl = tl + br - tr;
    vec2 positions[4] = vec2[](tl, bl, tr, br);

	gl_Position = u_projection * vec4(positions[gl_VertexID % 4], 0, 1);

    ivec2 atlas_size = textureSize(u_texture, 0);
    uint glyph_space = u_glyph_size + u_glyph_margin;
    uint columns = uint(atlas_size.x) / glyph_space;

    uint col = a_glyph_index % columns;
    uint row = a_glyph_index / columns;
	
    vec2 base_pixel = vec2(float(col * glyph_space), float(row * glyph_space));
    v_tex_coord = (base_pixel + 0.5 + OFFSETS[gl_VertexID % 4] * float(glyph_space - 1)) / vec2(atlas_size);
    v_color = a_color;
}
