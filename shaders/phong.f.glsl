#version 410

struct Light {
    vec4 pos;
    vec4 color;
};

layout(std140) uniform lights_buf {
    Light lights[LIGHT_COUNT];
};

layout(std140) uniform transform {
    mat4 model;
    mat4 view;
    mat4 proj;
    mat4 shadow_mat;
    vec4 eye_pos;
};

layout(std140) uniform display {
    vec4 ambient;
    vec4 material_spec;
    vec4 material_diff;
};

#ifdef SHADOWS
uniform sampler2DShadow shadow_depth;
#endif

in vec3 I_POS;
in vec3 I_NORM;

out vec4 f_color;

void main() {
    vec3 total = vec3(0, 0, 0);

    for (int i = 0; i < LIGHT_COUNT; i++) {
        vec3 ldir = lights[i].pos.xyz - I_POS;
        float distance = length(ldir);
        ldir /= distance;

        vec3 vdir = normalize(eye_pos.xyz - I_POS);

        vec3 color = (lights[i].color.rgb * lights[i].color.a) / (distance * distance);

        float diff = dot(I_NORM, ldir);
        diff = clamp(diff, 0, 1);
        vec3 add = color * material_diff.rgb * diff;

        vec3 halfway = normalize(ldir + vdir);
        float ndoth = dot(I_NORM, halfway);
        ndoth = clamp(ndoth, 0, 1);
        float spec = pow(ndoth, material_spec.w);

        add += color * material_spec.rgb * spec;

        #ifdef SHADOWS
        if (i == 0) {
            vec4 light_p = shadow_mat * vec4(I_POS, 1);
            vec3 s = light_p.xyz / light_p.w * 0.5 + 0.5;
            float d = texture(shadow_depth, s);
            add *= vec3(d);
        }
        #endif

        total += add;
    }

    total += material_diff.rgb * ambient.rgb;

    f_color = vec4(total, 1);
}