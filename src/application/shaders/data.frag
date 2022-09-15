#version 300 es

precision mediump float;

#define M_PI 3.1415926535898

in vec3 fragPosition;
in vec4 fragColor;

out vec4 outColor;

void main() {
    outColor = fragColor;
}
