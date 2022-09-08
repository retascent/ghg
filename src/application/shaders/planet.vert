#version 300 es

#define M_PI 3.14159265358979
// 32384626433832795

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;

uniform sampler2D s_textureMap;
uniform sampler2D s_colorMap;

uniform float u_terrainScale;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 fragPosition;
out vec3 fragNormal;
out vec2 fragSamplePosition;
out vec4 fragColor;

vec2 pointToUv(vec3 pointOnSphere) {
    float u = clamp(0.5 + atan(pointOnSphere.x, pointOnSphere.z) / 2.0 / M_PI, 0.0, 1.0);
    float v = clamp(0.5 + asin(pointOnSphere.y) / M_PI, 0.0, 1.0);
    return vec2(u, v);
}

// vec3 coordinateToPoint(vec2 coordinate) {
//     float y = sin(coordinate.x);
//     float r = cos(coordinate.x);
//     float x = sin(coordinate.y) * r;
//     float z = -cos(coordinate.y) * r;
//     return vec3(x, y, z);
// }

//bool isWater(vec2 texturePoint) {
//    vec4 color = texture(s_colorMap, texturePoint);
//    return color.b > color.r * 1.3 && color.b > color.g;
//}

void main() {
    vec2 texturePoint = pointToUv(normalize(position));
    float terrainValue = texture(s_textureMap, texturePoint).r;

    vec3 scaled_position;
//    if (isWater(texturePoint)) {
//        float positionScale = 1.0 - u_terrainScale / 2.0;
//        scaled_position = position * positionScale;
//    } else {
        float positionScale = 1.0 + (terrainValue * u_terrainScale) - u_terrainScale / 2.0;
        scaled_position = position * positionScale;
//    }

    gl_Position = u_projection * u_view * u_model * vec4(scaled_position, 1.0);

    fragPosition = vec3(u_model * vec4(scaled_position, 1.0));
    fragNormal = mat3(transpose(inverse(u_model))) * normal; // TODO: Inverse is very slow

    fragSamplePosition = texturePoint;
    fragColor = color;
}
