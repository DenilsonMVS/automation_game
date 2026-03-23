
#version 460 core

uniform vec4 u_color;
uniform vec2 u_view_min;
uniform vec2 u_view_max;
uniform vec2 u_dimensions;

in vec2 v_coord;
out vec4 color;

layout(std430, binding = 0) buffer GraphData {
    float y_values[];
};

float f(float x) {
    return x * x;
}

float get_y_from_buffer(float normalized_x) {
    uint count = y_values.length();
    
    if (count == 0) return 0.0;
    if (count == 1) return y_values[0];

    float float_idx = normalized_x * float(count - 1);
    
    uint i0 = max(0, uint(floor(float_idx)));
    uint i1 = min(i0 + 1, count - 1);

    float t = fract(float_idx);

    return mix(y_values[i0], y_values[i1], t);
}

void main() {
    vec2 point_coord = mix(u_view_min, u_view_max, v_coord);
    float buffer_y = get_y_from_buffer(v_coord.x);

    float dist_v = point_coord.y - buffer_y;
    float f_prime = dFdxFine(dist_v) / dFdxFine(point_coord.x);
    float dist_perp = abs(dist_v) / sqrt(f_prime * f_prime + 1.0);
    float view_height = u_view_max.y - u_view_min.y;
    float thickness = (0.015 / u_dimensions.y) * view_height;
    float half_thick = thickness * 0.5;
    float edge_softness = fwidthFine(dist_perp);
    float fill_mask = (point_coord.y < buffer_y) ? 0.1 : 0.0;
    float line_alpha = smoothstep(half_thick, half_thick - edge_softness, dist_perp);
    float final_alpha = max(line_alpha, fill_mask);

    color = vec4(u_color.rgb, u_color.a * final_alpha);
}
