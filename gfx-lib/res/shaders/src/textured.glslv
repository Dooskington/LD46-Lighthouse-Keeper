#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inUv;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragUv;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 model;
    mat4 projection;
} ubo;

void main() {
    fragColor = inColor;
    fragUv = inUv;
    
    gl_Position = ubo.projection * ubo.view * ubo.model * vec4(inPosition, 1.0);

    // Vulkan expects our Z range to be [0, 1] instead of [-1, 1],
    // so we must correct the final Z to be this range.
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}