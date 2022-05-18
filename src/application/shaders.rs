use web_sys::{WebGl2RenderingContext, WebGlProgram};
use crate::render_core::shader;

pub fn get_shaders(context: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
    let vert_shader = shader::compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

        #define M_PI 3.1415926535897932384626433832795

        layout(location = 0) in vec3 position;
        layout(location = 1) in vec3 normal;
        layout(location = 2) in vec4 color;

        uniform sampler2D s_textureMap;

        uniform float u_terrainScale;

        uniform mat4 u_model;
        uniform mat4 u_view;
        uniform mat4 u_projection;

        out vec3 fragPosition;
        out vec3 fragNormal;
        out vec4 fragColor;

        vec2 pointToUv(vec3 pointOnSphere) {
            float u = 0.5 + atan(pointOnSphere.x, pointOnSphere.z) / 2.0 / M_PI;
            float v = 0.5 + asin(pointOnSphere.y) / M_PI;
            return vec2(u, v);
        }

        // vec3 coordinateToPoint(vec2 coordinate) {
        //     float y = sin(coordinate.x);
        //     float r = cos(coordinate.x);
        //     float x = sin(coordinate.y) * r;
        //     float z = -cos(coordinate.y) * r;
        //     return vec3(x, y, z);
        // }

        void main() {
            vec2 texturePoint = pointToUv(position);
            float terrainValue = texture(s_textureMap, texturePoint).r;
            float positionScale = 1.0 + (terrainValue * u_terrainScale) - u_terrainScale / 2.0;

            vec3 scaled_position = position * positionScale;

            gl_Position = u_projection * u_view * u_model * vec4(scaled_position, 1.0);

            fragPosition = vec3(u_model * vec4(scaled_position, 1.0));
            fragNormal = mat3(transpose(inverse(u_model))) * normal; // TODO: Inverse is very slow

            if (terrainValue != 0.0) {
                float reproportioned = min(1.0, max(0.0, terrainValue * 1.3 + 0.1));
                vec4 mappedColor = mix(vec4(0.0, 0.0, 0.0, 1.0), vec4(1.0, 1.0, 1.0, 1.0), reproportioned);
                fragColor = mix(color, mappedColor, 0.9);
            } else {
                fragColor = mix(color, vec4(0.2, 0.2, 0.7, 1.0), 0.9);
            }
        }
        "##,
    )?;

    let frag_shader = shader::compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es

        precision mediump float;

        in vec3 fragPosition;
        in vec3 fragNormal;
        in vec4 fragColor;

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

        void main() {
            vec3 lightDir = normalize(u_lightPosition - fragPosition);
            vec3 norm = normalize(fragNormal);

            vec3 totalLightColor = getAmbientLight()
                + getDiffuseLight(lightDir, norm)
                + getSpecularLight(lightDir, norm);
            outColor = fragColor * vec4(totalLightColor, 1.0);
        }
        "##,
    )?;

    shader::link_program(&context, &vert_shader, &frag_shader)
}
