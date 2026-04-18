#version 460 core

out vec4 color;

in vec2 v_tex_coord;
in vec4 v_color;
uniform sampler2D u_texture;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = texture(u_texture, v_tex_coord).rgb;
    float dist = median(msd.r, msd.g, msd.b);

    float edge_width = fwidth(dist) * 0.5;
    float coverage = smoothstep(0.5 - edge_width, 0.5 + edge_width, dist);

    color = v_color * vec4(1, 1, 1, coverage);
}
