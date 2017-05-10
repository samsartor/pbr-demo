#version 410

uniform sampler2D value;

layout(std140) uniform live {
    vec4 eye_pos;
    vec4 ambient;
    float gamma;
    float exposure;
    float time;
};

in vec2 v_pos;
out vec4 f_color;

vec3 to_ldr(vec3 lum) {
    vec3 color = vec3(1.0) - exp(-lum * exposure);
    return pow(color, vec3(1.0 / gamma));
}

void main() {
    f_color = texture(value, v_pos);
    f_color.xyz = to_ldr(f_color.xyz);
} 