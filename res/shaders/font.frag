#version 460 core

out vec4 color;

in vec2 v_tex_coord;
in vec4 v_color;

uniform sampler2D u_texture;
uniform float u_px_range;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

float screen_px_range() {
    vec2 unit_range = vec2(u_px_range) / vec2(textureSize(u_texture, 0));
    vec2 screen_tex_size = vec2(1.0) / fwidth(v_tex_coord);
    return max(0.5 * dot(unit_range, screen_tex_size), 1.0);
}

void main() {
    vec3 msd = texture(u_texture, v_tex_coord).rgb;
    float sd = median(msd.r, msd.g, msd.b);
    
    float screen_px_distance = screen_px_range() * (sd - 0.5);
    float coverage = clamp(screen_px_distance + 0.5, 0.0, 1.0);

    color = v_color * vec4(1.0, 1.0, 1.0, coverage);
}
