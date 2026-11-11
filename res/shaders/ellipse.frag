
#version 460 core

in vec4 v_color;
in vec2 v_coord;

out vec4 color;

void main() {
    float dist = length(v_coord);
    float delta = fwidth(dist);
    float alpha = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, dist);
    
    color = vec4(v_color.rgb, v_color.a * alpha);
}
