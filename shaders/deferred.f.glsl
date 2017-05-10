#version 410

in vec3 v_pos;
in vec3 v_norm;
in vec2 v_tex;
in vec3 v_tan;
in vec3 v_bitan;

out vec4 layer_a;
out vec4 layer_b;

uniform sampler2D normal_tex;

void main() {
    vec3 normal_map = texture(normal_tex, v_tex).rgb * 2 - 1;

    // vec3 norm = mat3(v_tan, v_bitan, v_norm) * normal_map;
    vec3 norm = v_norm;

    layer_a = vec4(v_pos.xyz, norm.x);
    layer_b = vec4(norm.yz, v_tex.xy);
}