#version 410

in vec3 a_pos;
out vec2 v_pos;

void main() {
    v_pos = (a_pos.xy + 1) / 2;
    gl_Position = vec4(a_pos, 1);
}