#version 460 core

layout(location = 0, index = 0) out vec4 out_color;
layout(location = 0, index = 1) out vec4 out_subpixel_alpha;

in vec2 v_tex_coord;
in vec4 v_color;
uniform sampler2D u_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

float get_dist(vec2 uv) {
    vec3 val = texture(u_texture, uv).rgb;
    return median(val.r, val.g, val.b);
}

void main() {
    vec3 msd = texture(u_texture, v_tex_coord).rgb;
    float dist = median(msd.r, msd.g, msd.b);

    float dx = dFdx(dist);

    vec3 subpixel_dist = vec3(
        dist - dx * 0.333,
        dist,
        dist + dx * 0.333
    );

    float edge_width = fwidth(dist) * 0.5;
    vec3 coverage = smoothstep(0.5 - edge_width, 0.5 + edge_width, subpixel_dist);

    out_color = vec4(v_color.rgb * coverage, 1.0) * v_color.a;
    out_subpixel_alpha = vec4(coverage, 1.0) * v_color.a;
}
