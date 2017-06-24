#version 150 core

uniform usampler2D t_Canvas;
uniform sampler2D t_Labels;

uniform float i_Exp;
uniform vec2 i_Size;

in vec2 v_Uv;
out vec4 Target0;

void main() {
    ivec2 pos = ivec2(v_Uv * i_Size);
    uint u = texelFetch(t_Canvas, pos, 0).r;
    float label = 1.0;
    if (pos.y < 50) {
        label = 1.0 - texelFetch(t_Labels, pos, 0).r;
    }
    float v = float(u);
    vec3 blue = vec3(-0.005, -0.01, -0.002);
    vec3 color = exp(blue * v * i_Exp) * label;
    
    Target0 = vec4(color.r, color.g, color.b, 1.0);
}
