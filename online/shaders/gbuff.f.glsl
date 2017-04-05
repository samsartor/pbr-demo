#version 410

in vec3 I_POS;
in vec3 I_NORM;
in vec2 I_TEX;

layout(location = 0) out vec4 layera;
layout(location = 1) out vec4 layerb;

void main() {
    layera = vec4(I_POS.xyz, I_NORM.x);
    layerb = vec4(I_NORM.yz, I_TEX.xy);
}