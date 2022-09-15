#version 300 es

#define M_PI 3.1415926535898

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;

uniform sampler2D s_dataMap;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

uniform vec3 u_dataMinValues;
uniform vec3 u_dataMaxValues;

uniform float u_dataMinRadius;
uniform float u_dataMaxRadius;

uniform float u_dataScaleMultiplier;

out vec3 fragPosition;
out vec3 fragNormal;
out vec4 fragColor;

// TODO: Library shader
vec2 pointToUv(vec3 pointOnSphere) {
    float u = clamp(0.5 + atan(pointOnSphere.x, pointOnSphere.z) / 2.0 / M_PI, 0.0, 1.0);
    float v = clamp(0.5 + asin(pointOnSphere.y) / M_PI, 0.0, 1.0);
    return vec2(u, v);
}

// Re-maps the data from the texture using the metadata
vec3 channelValues(vec2 texturePoint) {
    vec3 channels = texture(s_dataMap, texturePoint).rgb;
    vec3 ranges = u_dataMaxValues - u_dataMinValues;
    return (channels * ranges) + u_dataMinValues;
}

void main() {
    int vertexIsMax = (gl_VertexID / 2) % 2;

    float displayRange = u_dataMaxRadius - u_dataMinRadius;

    vec2 texturePoint = pointToUv(normalize(position));
    vec3 dataRealValue = channelValues(texturePoint);

    float dataAbsMin = min(u_dataMinValues.r, u_dataMinValues.g);
    float dataAbsMax = max(u_dataMaxValues.r, u_dataMaxValues.g);
    float dataAbsRange = dataAbsMax - dataAbsMin;

    float dataProportion = (dataRealValue.g - dataRealValue.r) / dataAbsRange;
    float dataHeight = (dataProportion * displayRange);

    // TODO: Alpha blending
    fragColor = vec4(0.8, 0.0, 0.0, 0.8);
    if (dataHeight < 0.0) {
        fragColor = vec4(0.0, 0.8, 0.0, 0.8);
        dataHeight *= -1.0;
    }

    float valueScale = u_dataMinRadius;
    if (vertexIsMax == 0) {
        valueScale = u_dataMinRadius;
    } else {
        valueScale = u_dataScaleMultiplier * dataHeight + u_dataMinRadius;
    }
    vec3 scaledPosition = normalize(position) * valueScale;

    gl_PointSize = 6.0;
    gl_Position = u_projection * u_view * u_model * vec4(scaledPosition, 1.0);

    fragPosition = vec3(u_model * vec4(scaledPosition, 1.0));
}
