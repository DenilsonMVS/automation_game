#version 460 core

layout(location = 0, index = 0) out vec4 out_color;
layout(location = 0, index = 1) out vec4 out_subpixel_alpha;

in vec2 v_tex_coord;
in vec3 v_color;
uniform sampler2D u_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

float get_dist(vec2 uv) {
    vec3 val = texture(u_texture, uv).rgb;
    return median(val.r, val.g, val.b);
}

void main() {
    vec2 dx = dFdx(v_tex_coord) * (1.0 / 3.0);
    vec2 dy = dFdy(v_tex_coord) * 0.25;

    float r_dist = (get_dist(v_tex_coord - dx - dy) + get_dist(v_tex_coord - dx + dy)) * 0.5;
    float g_dist = (get_dist(v_tex_coord - dy)      + get_dist(v_tex_coord + dy))      * 0.5;
    float b_dist = (get_dist(v_tex_coord + dx - dy) + get_dist(v_tex_coord + dx + dy)) * 0.5;

    float delta_x = abs(b_dist - r_dist) * 0.5;
    float delta_y = abs(get_dist(v_tex_coord - dy) - get_dist(v_tex_coord + dy));
    float unit = (delta_x + delta_y) * 1.5;

    vec3 coverage = vec3(
        smoothstep(0.5 - unit, 0.5 + unit, r_dist),
        smoothstep(0.5 - unit, 0.5 + unit, g_dist),
        smoothstep(0.5 - unit, 0.5 + unit, b_dist)
    );

    out_color = vec4(v_color.rgb * coverage, 1.0);
    out_subpixel_alpha = vec4(coverage, 1.0);
}
