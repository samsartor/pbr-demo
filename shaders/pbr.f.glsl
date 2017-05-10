#version 410

uniform sampler2D layer_a;
uniform sampler2D layer_b;
uniform sampler2D albedo_tex;
uniform sampler2D metalness_tex;
uniform sampler2D roughness_tex;

layout(std140) uniform live {
    vec4 eye_pos;
    float gamma;
    float exposure;
    float time;
};

in vec2 v_pos;

out vec4 f_color;

const float PI = 3.14159265359;

const int LIGHT_COUNT = 5;
const vec3 ambient = vec3(0.015, 0.015, 0.025) * 1.5;

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
    
    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    
    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
    
    return ggx1 * ggx2;
}

vec3 to_ldr(vec3 lum) {
    vec3 color = vec3(1.0) - exp(-lum * exposure);
    return pow(color, vec3(1.0 / gamma)); // Not sure why this over-gammas it
}

void main() {
    vec4 a = texture(layer_a, v_pos);
    vec4 b = texture(layer_b, v_pos);

    vec3 pos = a.xyz;
    vec3 norm = vec3(a.w, b.xy);
    vec2 tex = b.zw;

    if (dot(norm, norm) < 0.001) {
        f_color = vec4(to_ldr(ambient), 0);
        return;
    }

    vec3 albedo = pow(texture(albedo_tex, tex).rgb, vec3(2.2));
    float roughness = texture(roughness_tex, tex).r;
    float metalness = texture(metalness_tex, tex).r;

    vec3 lum = vec3(0, 0, 0);

    // ONCE
    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, albedo, metalness);

    vec3 N = normalize(norm);
    vec3 V = normalize(eye_pos.xyz - pos);

    // PER LIGHT

    for (int light_i = 0; light_i < LIGHT_COUNT; light_i++) {
        float light_angle = PI * light_i * 2.0 / LIGHT_COUNT;
        light_angle += 0.333 * time;
        vec3 light_pos = vec3(9 * sin(light_angle), sin(time * 0.6) * 3, 9 * cos(light_angle));
        vec3 light_color = vec3(1.0, 0.9, 0.5) * 50.;

        vec3 L = normalize(light_pos - pos);
        vec3 H = normalize(V + L);
        float distance    = length(light_pos - pos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     = light_color * attenuation;        
        
        // cook-torrance brdf
        float NDF = DistributionGGX(N, H, roughness);        
        float G   = GeometrySmith(N, V, L, roughness);      
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);       
        
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metalness;     
        
        vec3 nominator    = NDF * G * F;
        float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; 
        vec3 brdf = nominator / denominator;
            
        // add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);                
        lum += (kD * albedo / PI + brdf) * radiance * NdotL; 
    }

    // AMBIENT
    lum += ambient * albedo; // * ao;

    // OUT
    f_color = vec4(to_ldr(lum), 1);
} 