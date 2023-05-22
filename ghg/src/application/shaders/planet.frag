#version 300 es

precision mediump float;

#include <application/shaders/channels.glsl>
#include <application/shaders/color.glsl>
#include <application/shaders/pointmapping.glsl>
#include <application/shaders/math.glsl>

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

// Data parameters
const int NUM_MAPS_PER_YEAR = 3;
const int NUM_CHANNELS_IN_MAP = 4;
uniform int u_dataMonth;
uniform sampler2D s_dataMap;
uniform mat3x4 u_dataMinValues; // TOOD: float for year- or data-length min/max
uniform mat3x4 u_dataMaxValues;

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

vec4 getDataColor() {
    int mapIndex = u_dataMonth / NUM_MAPS_PER_YEAR;
    int channelInMap = u_dataMonth % NUM_CHANNELS_IN_MAP;

    vec4 minValues = u_dataMinValues[mapIndex];
    vec4 maxValues = u_dataMaxValues[mapIndex];

    vec2 texturePoint = pointToUv(normalize(fragPosition));
    vec4 dataRealValue = channelValues(s_dataMap, texturePoint, minValues, maxValues);

    vec4 dataRange = maxValues - minValues;

    vec4 dataProportion = (dataRealValue - minValues) / dataRange;
    float truncateColorSpace = 0.9;
    vec4 truncatedProportion = (vec4(1.0) - dataProportion) * truncateColorSpace;

    float channelValue = channelIndex(truncatedProportion, channelInMap);

    vec3 withinColorSpace = hsl2rgb(vec3(channelValue, 1.0, 0.5));
    return vec4(withinColorSpace, 1.0);

    //    if (channelInMap == 0) {
    //        return vec4(vec3(dataRealValue.r), 1.0);
    //    } else if (channelInMap == 1) {
    //        return vec4(vec3(dataRealValue.g), 1.0);
    //    } else if (channelInMap == 2) {
    //        return vec4(vec3(dataRealValue.b), 1.0);
    //    } else { // if (channelInMap == 3) {
    //        return vec4(vec3(dataRealValue.a), 1.0);
    //    }
}

void main() {
    vec3 lightDir = normalize(u_lightPosition - fragPosition);
    vec3 norm = normalize(fragNormal);

    vec3 totalLightColor = getAmbientLight()
    + getDiffuseLight(lightDir, norm)
    + getSpecularLight(lightDir, norm);

    vec4 terrainColor = getTerrainColor();
    vec4 dataColor = getDataColor();

    vec4 surfaceColor = mix(terrainColor, dataColor, 0.7);

    outColor = surfaceColor * vec4(totalLightColor, 1.0);
}
