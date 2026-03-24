
#version 460 core

in vec2 v_coord;
out vec4 color;

layout(std430, binding = 0) buffer GraphData {
    vec2 center;
    vec2 dimensions;
    vec4 primary_color;
    vec2 view_min;
    vec2 view_max;
    float y_values[];
};

float dist_to_segment(vec2 p, vec2 a, vec2 b) {
    vec2 pa = p - a, ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

void main() {
    vec2 p = mix(view_min, view_max, v_coord);
    uint count = y_values.length();

    float horizontal_t = clamp((p.x - view_min.x) / (view_max.x - view_min.x), 0.0, 1.0);
    float float_idx = horizontal_t * float(count - 1);
    uint i0 = uint(floor(float_idx));
    uint i1 = min(i0 + 1, count - 1);
    float buffer_y_at_x = mix(y_values[i0], y_values[i1], fract(float_idx));

    float min_dist = 1e10;
    for (uint i = 0; i + 1 < count; ++i) {
        vec2 a = vec2(mix(view_min.x, view_max.x, float(i) / float(count - 1)), y_values[i]);
        vec2 b = vec2(mix(view_min.x, view_max.x, float(i + 1) / float(count - 1)), y_values[i + 1]);
        min_dist = min(min_dist, dist_to_segment(p, a, b));
    }

    float view_height = abs(view_max.y - view_min.y);
    float thickness = 0.01 * view_height;
    float line_alpha = smoothstep(thickness * 0.5, thickness * 0.5 - fwidth(min_dist), min_dist);
    float fill_mask = (p.y < buffer_y_at_x) ? 0.1 : 0.0;

    color = vec4(primary_color.rgb, primary_color.a * max(line_alpha, fill_mask));
}