#version 410

uniform sampler2D layera;
uniform sampler2D layerb;
uniform sampler2D albedo_tex;
uniform sampler2D roughness_tex;
uniform vec3 camera_pos;
uniform float time;

in vec2 v_pos;

layout(location = 0) out vec4 f_color;

const float PI = 3.14159265359;
const int LIGHT_COUNT = 5;

void main() {
    vec4 a = texture(layera, v_pos);
    vec4 b = texture(layerb, v_pos);

    vec3 pos = a.xyz;
    vec3 norm = vec3(a.w, b.xy);
    vec2 tex = b.zw;

    if (dot(norm, norm) < 0.001) {
        f_color = vec4(0.015, 0.015, 0.02, 0);
        return;
    }

    vec3 albedo = texture(albedo_tex, tex).rgb;
    float roughness = texture(roughness_tex, tex).r;
    float hard = (1 / (pow(roughness, 4) + 0.001));

    // ONCE
    vec3 N = normalize(norm);
    vec3 V = normalize(camera_pos - pos);

    vec3 lum = vec3(0, 0, 0);

    // PER LIGHT

    for (int light_i = 0; light_i < LIGHT_COUNT; light_i++) {
        float light_angle = PI * light_i * 2.0 / LIGHT_COUNT;
        vec3 light_pos = vec3(5 * sin(light_angle), sin(time * 0.6) * 4, 5 * cos(light_angle));
        vec3 light_color = vec3(48, 40, 24) * 0.6;

        vec3 L = normalize(light_pos - pos);
        vec3 H = normalize(V + L);
        float distance    = length(light_pos - pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     = light_color * attenuation;

        float lambert = dot(N, V);
        if (lambert > 0) {
            float spec = pow(clamp(dot(N, H), 0, 1), hard);
            lum += radiance * (albedo * lambert + vec3(spec));
        }
    }

    // AMBIENT
    lum += vec3(0.015, 0.015, 0.02) * albedo;

    // OUT
    lum = clamp(lum, 0, 1);
    f_color = vec4(lum, 1);
} 