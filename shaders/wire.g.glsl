#version 410

layout(triangles) in;
layout(triangle_strip, max_vertices = 3) out;

in vec3 I_POS[];
in vec3 I_NORM[];

out vec3 g_pos;
out vec3 g_norm;
out vec3 g_alt;

uniform int screen_width;
uniform int screen_height;

vec2 viewport(vec4 pos) {
    vec2 transformed = (pos / pos.w).xy;
    transformed.x += 1;
    transformed.y += 1;
    transformed.x *= 0.5 * screen_width;
    transformed.y *= 0.5 * screen_height;
    return transformed;
}

void main() {
    vec2 p0 = viewport(gl_in[0].gl_Position);
    vec2 p1 = viewport(gl_in[1].gl_Position);
    vec2 p2 = viewport(gl_in[2].gl_Position);

    float a = length(p1 - p2);
    float b = length(p2 - p0);
    float c = length(p1 - p0);
    float alpha = acos((b*b + c*c - a*a) / (2.0*b*c));
    float beta = acos((a*a + c*c - b*b) / (2.0*a*c));
    float ha = abs(c * sin(beta));
    float hb = abs(c * sin(alpha));
    float hc = abs(b * sin(alpha));

    g_pos = I_POS[0];
    g_norm = I_NORM[0];
    g_alt = vec3(ha, 0, 0);
    gl_Position = gl_in[0].gl_Position;
    EmitVertex();

    g_pos = I_POS[1];
    g_norm = I_NORM[1];
    g_alt = vec3(0, hb, 0);
    gl_Position = gl_in[1].gl_Position;
    EmitVertex();

    g_pos = I_POS[2];
    g_norm = I_NORM[2];
    g_alt = vec3(0, 0, hc);
    gl_Position = gl_in[2].gl_Position;
    EmitVertex();

    EndPrimitive();
}