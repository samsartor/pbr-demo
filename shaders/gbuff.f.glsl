#version 410

in vec3 I_POS;
in vec3 I_NORM;
in vec2 I_TEX;
in vec3 I_TAN;
in vec3 I_BITAN;

layout(location = 0) out vec4 layera;
layout(location = 1) out vec4 layerb;

uniform sampler2D normal_tex;
uniform float norm_map_strength;

void main() {
    vec3 normal_map = texture(normal_tex, I_TEX).rgb * 2 - 1;

    vec3 norm = mat3(I_TAN, I_BITAN, I_NORM) * normal_map;
    norm = normalize(mix(I_NORM, norm, norm_map_strength));

    layera = vec4(I_POS.xyz, norm.x);
    layerb = vec4(norm.yz, I_TEX.xy);
}