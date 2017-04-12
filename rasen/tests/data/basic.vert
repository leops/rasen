#version 310 es

struct _9
{
    mat4 _0;
    mat4 _1;
    mat4 _2;
};

layout(location = 0) in vec3 _25;
layout(location = 0) out vec4 _34;
layout(location = 1) in vec3 _38;
layout(location = 1) out vec4 _47;
layout(location = 2) in vec2 _50;
layout(location = 2) out vec2 _53;

void main()
{
    _34 = ((_11._0 * _11._1) * _11._2) * vec4(_25, 1.0);
    _47 = _11._2 * vec4(_38, 0.0);
    _53 = _50;
    gl_Position.z = 2.0 * gl_Position.z - gl_Position.w;
}

