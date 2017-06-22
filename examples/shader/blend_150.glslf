#version 150 core

// grayscale
uniform sampler2D t_Canvas;

uniform float i_Exp;

in vec2 v_Uv;
out vec4 Target0;

void main() {
    vec4 v = texture2D(t_Canvas, v_Uv);

    Target0 = vec4(1.0 - v.b, 1.0 - v.r, 1.0 - v.r, 1.0);
}
