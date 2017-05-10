#version 410

const float PI = 3.14159265359;

uniform sampler2D layer_a;
uniform sampler2D layer_b;
uniform sampler2D albedo_tex;
uniform sampler2D metalness_tex;
uniform sampler2D roughness_tex;
uniform sampler2DShadow shadow_depth;

layout(std140) uniform live {
    vec4 eye_pos;
    float gamma;
    float exposure;
    float time;
};

layout(std140) uniform light {
    mat4 light_matrix;
    vec4 light_pos;
    vec4 light_color;
    vec4 ambient;
};

in vec2 v_pos;
out vec4 f_color;

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float distributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness * roughness;
    a = a * a;
    float n_dot_h = max(dot(N, H), 0.0);
    float n_dot_h2 = n_dot_h * n_dot_h;
    
    float denom = (n_dot_h2 * (a - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return a / denom;
}

float geometrySchlickGGX(float n_dot_v, float roughness)
{
    float rough_more = (roughness + 1.0);
    float k = (rough_more * rough_more) / 8.0;

    float denom = n_dot_v * (1.0 - k) + k;
    
    return n_dot_v / denom;
}

float geometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float n_dot_v = max(dot(N, V), 0.0);
    float n_dot_l = max(dot(N, L), 0.0);
    float ggx2 = geometrySchlickGGX(n_dot_v, roughness);
    float ggx1 = geometrySchlickGGX(n_dot_l, roughness);
    
    return ggx1 * ggx2;
}

void main() {
    vec4 a = texture(layer_a, v_pos);
    vec4 b = texture(layer_b, v_pos);

    vec3 pos = a.xyz;
    vec3 norm = vec3(a.w, b.xy);
    vec2 tex = b.zw;

    vec3 back = ambient.rgb * ambient.a;

    if (dot(norm, norm) < 0.001) {
        f_color = vec4(back, 0);
        return;
    }

    vec3 albedo = pow(texture(albedo_tex, tex).rgb, vec3(2.2));
    float roughness = texture(roughness_tex, tex).r;
    float metalness = texture(metalness_tex, tex).r;

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, albedo, metalness);

    vec3 N = normalize(norm);
    vec3 V = normalize(eye_pos.xyz - pos);

    vec3 lpos = light_pos.xyz;

    vec3 L = normalize(lpos - pos);
    vec3 H = normalize(V + L);
    float dist = length(lpos - pos);
    vec3 radiance = light_color.rgb * light_color.a / (dist * dist);
    
    // brdf
    float NDF = distributionGGX(N, H, roughness);        
    float G = geometrySmith(N, V, L, roughness);      
    vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);       
    
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metalness;     
    
    vec3 nominator = NDF * G * F;
    float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; 
    vec3 brdf = nominator / denominator;
        
    // add to outgoing radiance Lo
    float n_dot_l = max(dot(N, L), 0.0);                
    vec3 lum = (kD * albedo / PI + brdf) * radiance * n_dot_l;

    // shadows
    vec4 light_p = light_matrix * vec4(pos, 1);
    vec3 s = light_p.xyz / light_p.w * 0.5 + 0.5;
    float d = texture(shadow_depth, s);
    lum *= vec3(d);

    // AMBIENT
    lum += back * albedo; // * ao;

    // OUT
    f_color = vec4(lum, 1);
} 