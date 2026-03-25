#version 460 core

in vec2 v_tex_coord;
in vec3 v_color;
in vec2 v_sample;

uniform sampler2D u_texture;

out vec4 color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

float sample_sdf(float a) {
    float sharpness = 2.0;
    return clamp((a - 0.5) * sharpness + 0.5, 0.0, 1.0);
}

float sample_msdf(vec3 rgb) {
    float d = median(rgb.r, rgb.g, rgb.b) - 0.5;
    float range = fwidth(d);
    return smoothstep(-range, range, d);
}

void main() {
    vec4 s = texture(u_texture, v_tex_coord);
    float sdf = sample_sdf(s.a);
    float msdf = sample_msdf(s.rbg);
    color = vec4(v_color, dot(v_sample, vec2(sdf, msdf)));
}
