#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragUv;

layout(location = 0) out vec4 target;

layout(set = 0, binding = 1) uniform texture2D colorMap;
layout(set = 0, binding = 2) uniform sampler colorSampler;

void main() {
    target = fragColor * texture(sampler2D(colorMap, colorSampler), fragUv);
}