#version 300 es

precision mediump float;

in vec3 fragPosition;
in vec4 fragColor;

out vec4 outColor;

void main() {
    outColor = fragColor;
}
