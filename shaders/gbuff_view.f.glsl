#version 410

uniform sampler2D layera;
uniform sampler2D layerb;
uniform vec2 pos_range;
uniform vec2 norm_range;
uniform vec2 tex_range;
uniform sampler2D albedo_tex;
uniform sampler2D metalness_tex;
uniform vec2 pix_size;

in vec2 v_pos;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 a = texture(layera, v_pos);
    vec4 b = texture(layerb, v_pos);

    vec3 pos = a.xyz;
    vec3 norm = vec3(a.w, b.xy);
    vec2 tex = b.zw;

    vec3 albedo = texture(albedo_tex, tex).rgb;
    float metalness = texture(metalness_tex, tex).r;

    if (dot(norm, norm) < 0.001) {
        f_color = vec4(0, 0, 0, 0);
        return;
    }

    float sep = v_pos.y;

    if (pos_range.x <= sep && sep < pos_range.y) {
        f_color = vec4(pos / 2 + 0.5, 1.0);
    } else if (norm_range.x <= sep && sep < norm_range.y) {
        f_color = vec4(norm / 2 + 0.5, 1.0);
    } else if (tex_range.x <= sep && sep < tex_range.y) {
        int c = int(tex.x * 20) + int(tex.y * 20);
        if (c % 2 == 0) f_color = vec4(albedo, 1.0);
        else f_color = vec4(metalness, metalness, metalness, 1.0);
    }
}