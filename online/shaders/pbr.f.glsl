#version 410

uniform sampler2D layera;
uniform sampler2D layerb;
uniform sampler2D albedo_tex;
uniform sampler2D metalness_tex;
uniform sampler2D roughness_tex;
uniform sampler2D normal_tex;
uniform vec3 camera_pos;

in vec2 v_pos;

layout(location = 0) out vec4 f_color;

const float PI = 3.14159265359;

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

    vec3 albedo = texture(albedo_tex, tex).rgb;
    float roughness = texture(roughness_tex, tex).r;
    float metalness = texture(metalness_tex, tex).r;
    vec3 normal_map = texture(normal_tex, tex).rgb * 2 - 1;

    vec3 lum;

    // PER LIGHT

    vec3 light_pos = vec3(5, 5, 5);
    vec3 light_color = vec3(64, 64, 64);

    vec3 N = normalize(norm);
    vec3 V = normalize(camera_pos - pos);
    vec3 L = normalize(light_pos - pos);
    vec3 H = normalize(V + L);
  
    float distance    = length(light_pos - pos);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance     = light_color * attenuation;

    vec3 F0 = vec3(0.04); 
    F0      = mix(F0, albedo, metalness);
    vec3 F  = fresnelSchlick(max(dot(H, V), 0.0), F0);

    float NDF = DistributionGGX(N, H, roughness);       
    float G   = GeometrySmith(N, V, L, roughness); 

    vec3 nominator    = NDF * G * F;
    float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; 
    vec3 brdf         = nominator / denominator; 

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
      
    kD *= 1.0 - metalness;
  
    float NdotL = max(dot(N, L), 0.0);        
    lum += (kD * albedo / PI + brdf) * radiance * NdotL;

    // AMBIENT
    lum += vec3(0.03) * albedo; // * ao;

    // HDR
    vec3 color = lum / (lum + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));


    // OUT
    f_color = vec4(color, 1);
} 