#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec3 fragNormal;

layout(location = 0) out vec3 outColor;


layout(set = 0, binding = 0) buffer MaterialParams {
    vec3 ambient;
	float shininess_exponent;
    vec3 diffuse;
    float optical_density;
    vec3 specular;
    float dissolve;
    vec3 emission;
    int illumination;
} materials_params;


void main() {
    // Lighting
	vec3 lightColor = vec3(1.0f,1.0f,1.0f); // white
	vec3 lightSource = vec3(0.0f, 0.0f, 1.0f); // light pos

	float diffuseStrength = max(0.0, dot(lightSource, fragNormal)); // dot - should interl per frag
	vec3 diffuse = diffuseStrength * lightColor; // diffuse color

	vec3 lighting = materials_params.ambient;
	lighting = materials_params.ambient * 0.0 + diffuse; // apply changes


    outColor = fragColor * lighting;
}
