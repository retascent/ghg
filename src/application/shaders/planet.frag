#version 300 es

precision mediump float;

in vec3 fragPosition;
in vec3 fragNormal;
in vec2 fragSamplePosition;
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
    float terrainValue = texture(s_textureMap, fragSamplePosition).r;
    vec4 mappedColor = texture(s_colorMap, fragSamplePosition);
    return mix(fragColor, mappedColor, 0.93);
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
