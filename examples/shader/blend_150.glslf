#version 150 core

// grayscale
uniform usampler2D t_Canvas;

uniform float i_Exp;
uniform vec2 i_Size;

in vec2 v_Uv;
out vec4 Target0;

void main() {
    uint u = texelFetch(t_Canvas, ivec2(v_Uv * i_Size), 0).r;
    float v = float(u);
    vec4 blue = vec4(0.995, 0.99, 0.998, 1.0);
    
    Target0 = pow(blue, vec4(v));
}
