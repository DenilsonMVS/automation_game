#version 460 core

in vec2 v_tex_coord;
in vec4 v_color;

uniform sampler2D u_texture; 

out vec4 color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 s = texture(u_texture, v_tex_coord).rgb;
    float d = median(s.r, s.g, s.b);

    float w = clamp((d - 0.45) * 2.0, 0.0, 1.0);
    w = w * w * (3.0 - 2.0 * w); 

    color = vec4(v_color.rgb, v_color.a * w);
}
