#version 410

struct Light {
    vec3 pos;
    vec3 color;
};

uniform Light lights[LIGHT_COUNT];
uniform vec3 ambient;

uniform vec3 material_spec;
uniform vec3 material_diff;
uniform float material_hard;

uniform vec3 cam_pos;

#ifdef WIRE
uniform vec3 line_color;
uniform float line_width;

uniform int show_wireframe;
#endif

in vec3 I_POS;
in vec3 I_NORM;

#ifdef WIRE
in vec3 g_alt;
#endif

out vec4 f_color;

void main() {
    vec3 total = vec3(0, 0, 0);

    for (int i = 0; i < LIGHT_COUNT; i++) {
        vec3 ldir = lights[i].pos - I_POS;
        float distance = length(ldir);
        ldir /= distance;

        vec3 vdir = normalize(cam_pos - I_POS);

        vec3 color = lights[i].color / (distance * distance);

        float diff = dot(I_NORM, ldir);
        diff = clamp(diff, 0, 1);
        total += color * material_diff * diff;

        vec3 halfway = normalize(ldir + vdir);
        float ndoth = dot(I_NORM, halfway);
        ndoth = clamp(ndoth, 0, 1);
        float spec = pow(ndoth, material_hard);
        total += color * material_spec * spec;
    }

    total += material_diff * ambient;

    #ifdef WIRE
    if (show_wireframe == 1) {
        float edge_dist = min(g_alt.x, min(g_alt.y, g_alt.z));
        float edge_blend = smoothstep(line_width - 1, line_width + 1, edge_dist);
        total = mix(line_color, total, edge_blend);
    }
    #endif

    f_color = vec4(total, 1);
}