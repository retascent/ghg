#version 300 es

precision mediump float;

#define M_PI 3.1415926535898

in vec3 fragPosition;
in vec3 fragNormal;
in vec4 fragColor;

uniform sampler2D s_textureMap;
uniform sampler2D s_colorMap;

out vec4 outColor;

// Lighting parameters
uniform float u_ambientStrength;
uniform vec3 u_ambientColor;

uniform vec3 u_lightPosition;
uniform vec3 u_lightColor;

uniform vec3 u_cameraPosition;
uniform float u_specularStrength;

vec2 pointToUv(vec3 pointOnSphere) {
    float u = clamp(0.5 + atan(pointOnSphere.x, pointOnSphere.z) / 2.0 / M_PI, 0.0, 1.0);
    float v = clamp(0.5 + asin(pointOnSphere.y) / M_PI, 0.0, 1.0);
    return vec2(u, v);
}

vec3 getAmbientLight() {
    return u_ambientStrength * u_ambientColor;
}

vec3 getDiffuseLight(vec3 lightDir, vec3 norm) {
    float diff = max(dot(norm, lightDir), 0.0);
    return diff * u_lightColor;
}

vec3 getSpecularLight(vec3 lightDir, vec3 norm) {
    const float SHININESS = 32.0;

    vec3 viewDir = normalize(u_cameraPosition - fragPosition);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), SHININESS);
    return u_specularStrength * spec * u_lightColor;
}

vec4 getTerrainColor() {
    // TODO: fragSamplePosition cannot be calculated in vertex shader and passed here as-is.
    // Because vertex values are interpolated, the small space on the "back" of the planet (where the texture
    // wraps) is a 1-vertex-wide gap that interpolates fragments across the whole planet texture.
    // The simplest solution may be to double up the vertices just along that back line, so they map cleanly.

    vec2 fragSamplePosition = pointToUv(normalize(fragPosition));

    float terrainValue = texture(s_textureMap, fragSamplePosition).r;
    vec4 mappedColor = texture(s_colorMap, fragSamplePosition);
    return mix(fragColor, mappedColor, 0.99);

    // Grayscale based on depth:
//    return mix(fragColor, vec4(terrainValue, terrainValue, terrainValue, 1.0), 0.93);
}

void main() {
    vec3 lightDir = normalize(u_lightPosition - fragPosition);
    vec3 norm = normalize(fragNormal);

    vec3 totalLightColor = getAmbientLight()
            + getDiffuseLight(lightDir, norm)
            + getSpecularLight(lightDir, norm);

    vec4 terrainColor = getTerrainColor();

    outColor = terrainColor * vec4(totalLightColor, 1.0);
}
