#version 450
#extension GL_EXT_buffer_reference : require

layout (location = 0) out vec3 outColor;
layout (location = 1) out vec3 outNormal;

struct Vertex {
	vec3 position; //considered as vec4
	uint uv_x;
	vec3 color;
	uint uv_y;
	vec3 normal;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer{
	Vertex vertices[];
};



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



//push constants block
layout( push_constant ) uniform constants
{
	mat4 render_matrix;
	VertexBuffer vertexBuffer;
} PushConstants;

void main()
{
	//load vertex data from device adress
	Vertex v = PushConstants.vertexBuffer.vertices[gl_VertexIndex];

	// Position
	gl_Position = PushConstants.render_matrix * vec4(v.position, 1.0f);

	// Color
	vec3 color = vec3(0.5,0.5,0.5);

	outNormal = v.normal;
	outColor = color;
}