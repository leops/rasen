#version 450

layout(location = 0) in vec3 _8;
layout(location = 0) out vec4 _26;

void main()
{
    _26 = clamp(dot(normalize(_8), vec3(0.300000011920928955078125, -0.5, 0.20000000298023223876953125)), 0.100000001490116119384765625, 1.0) * vec4(0.25, 0.625, 1.0, 1.0);
}

