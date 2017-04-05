#version 410

uniform sampler2D layera;
uniform sampler2D layerb;
uniform vec2 pos_range;
uniform vec2 norm_range;
uniform vec2 tex_range;

in vec2 v_pos;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 a = texture(layera, v_pos);
    vec4 b = texture(layerb, v_pos);

    vec3 pos = a.xyz;
    vec3 norm = vec3(a.w, b.xy);
    vec2 tex = b.zw;

    if (dot(norm, norm) < 0.001) {
        f_color = vec4(0, 0, 0, 0);
        return;
    }

    if (pos_range.x <= v_pos.x && v_pos.x < pos_range.y) {
        f_color = vec4(pos / 2 + 0.5, 1.0);
    } else if (norm_range.x <= v_pos.x && v_pos.x < norm_range.y) {
        f_color = vec4(norm / 2 + 0.5, 1.0);
    } else if (tex_range.x <= v_pos.x && v_pos.x < tex_range.y) {
        int c = int(tex.x * 20) + int(tex.y * 20);
        float v = (c % 2) * 0.5 + 0.3;
        f_color = vec4(v, v, v, 1.0);
    }
}