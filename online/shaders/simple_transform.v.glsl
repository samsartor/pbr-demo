#version 410
 
uniform mat4 model;

#ifdef VIEWPROJ
uniform mat4 proj;
uniform mat4 view;
#endif

in vec3 a_pos;
out vec3 v_pos;

#ifdef NORM
in vec3 a_nor;
out vec3 v_norm;
#endif

void main() {
    vec4 p = model * vec4(a_pos, 1);
    v_pos = p.xyz;

    #ifdef NORM
    vec4 n = normalize(model * vec4(a_nor, 0));
    v_norm = n.xyz;
    #endif

    #ifdef VIEWPROJ
    gl_Position = proj * view * p;
    #else
    gl_Position = p;
    #endif
}