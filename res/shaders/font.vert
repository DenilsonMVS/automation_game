#version 460 core

layout (location = 0) in ivec2 a_top_left; 
layout (location = 1) in ivec2 a_top_right;
layout (location = 2) in ivec2 a_bottom_right;
layout (location = 3) in vec3 a_color;
layout (location = 4) in uint a_glyph_index;

out vec2 v_tex_coord;
out vec3 v_color;
out vec2 v_sample;

uniform sampler2D u_texture;
uniform uint u_glyph_size;
uniform uint u_glyph_margin;
uniform float u_world_scale;
uniform vec2 u_screen_size;

// (size, start_x, start_y, cut_size)
uniform uvec4 u_sample_20;
uniform uvec4 u_sample_12;
uniform uvec4 u_sample_8;
uniform uvec4 u_sample_4;

const vec2 OFFSETS[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
	vec2 ratio = u_screen_size.x > u_screen_size.y
		? vec2(u_screen_size.y / u_screen_size.x, 1)
		: vec2(1, u_screen_size.x / u_screen_size.y);

    vec2 tl = vec2(a_top_left) * u_world_scale;
    vec2 tr = vec2(a_top_right) * u_world_scale;
    vec2 br = vec2(a_bottom_right) * u_world_scale;
    vec2 bl = tl + br - tr;
    vec2 positions[4] = vec2[](tl, bl, tr, br);

	vec2 tl_px_pos = (tl * ratio + 1) * u_screen_size * 0.5;
	vec2 rounded_tl_px = floor(tl_px_pos + 0.5);
	vec2 original_rounded_tl = (rounded_tl_px * 2 / u_screen_size - 1) / ratio;

	vec2 br_px_pos = (br * ratio + 1) * u_screen_size * 0.5;
	float size = length(tl_px_pos - br_px_pos);

	vec2 tl_diff = positions[gl_VertexID % 4] - tl;
	vec2 rounded = tl_diff + original_rounded_tl;

	gl_Position = vec4(rounded * ratio, 0, 1);

    ivec2 atlas_size = textureSize(u_texture, 0);
    uint glyph_space = u_glyph_size + u_glyph_margin;
    uint columns = uint(atlas_size.x) / glyph_space;
	uint tex_sample_size = size < 37 ? 20 : glyph_space;

    uint col = a_glyph_index % columns;
    uint row = a_glyph_index / columns;

	uint sample_px_size = glyph_space;
	uvec2 sample_px_start = uvec2(0, 0);
	v_sample = vec2(0, 1);

	if(size <= u_sample_4.w) {
		v_sample = vec2(1, 0);
		sample_px_size = u_sample_4.x;
		sample_px_start = u_sample_4.yz;
	} else if(size <= u_sample_8.w) {
		v_sample = vec2(1, 0);
		sample_px_size = u_sample_8.x;
		sample_px_start = u_sample_8.yz;
	} else if(size <= u_sample_12.w) {
		v_sample = vec2(1, 0);
		sample_px_size = u_sample_12.x;
		sample_px_start = u_sample_12.yz;
	} else if(size <= u_sample_20.w) {
		v_sample = vec2(1, 0);
		sample_px_size = u_sample_20.x;
		sample_px_start = u_sample_20.yz;
	}
	
    vec2 base_pixel = vec2(float(col * glyph_space), float(row * glyph_space));
    v_tex_coord = (base_pixel + sample_px_start + 0.5 + OFFSETS[gl_VertexID % 4] * float(sample_px_size - 1)) / vec2(atlas_size);
    v_color = a_color;
}
