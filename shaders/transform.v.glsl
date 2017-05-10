#version 410
 
layout(std140) uniform transform {
    mat4 model;
    mat4 view;
    mat4 proj;
};

in vec3 a_pos;
out vec3 v_pos;

#ifdef NORM
in vec3 a_nor;
out vec3 v_norm;
#endif

#ifdef TAN
in vec3 a_tan;
out vec3 v_tan;
in vec3 a_btn;
out vec3 v_bitan;
#endif

void main() {
    vec4 p = model * vec4(a_pos, 1);
    v_pos = p.xyz;

    #ifdef NORM
    v_norm = (model * vec4(a_nor, 0)).xyz;
    #endif

    #ifdef TAN
    v_tan = normalize((model * vec4(a_tan, 0)).xyz);
    v_bitan = normalize((model * vec4(a_btn, 0)).xyz);
    #endif

    #ifdef VIEWPROJ
    gl_Position = proj * view * p;
    #else
    gl_Position = p;
    #endif
}