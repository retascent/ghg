use web_sys::{WebGl2RenderingContext, WebGlProgram};
use crate::render_core::smart_uniform::SmartUniform;

pub struct LightParameters {
    pub ambient_strength: SmartUniform<f32>,
    pub specular_strength: SmartUniform<f32>,
    pub ambient_color: SmartUniform<nglm::Vec3>,
    pub light_color: SmartUniform<nglm::Vec3>,
    pub light_position: SmartUniform<nglm::Vec3>,
    pub camera_position: SmartUniform<nglm::Vec3>,
}

impl LightParameters {
    pub fn new(context: &WebGl2RenderingContext, program: &WebGlProgram) -> Self {
        let mut ambient_strength = SmartUniform::<f32>::new("u_ambientStrength", context.clone(), program.clone());
        let mut specular_strength = SmartUniform::<f32>::new("u_specularStrength", context.clone(), program.clone());
        let mut ambient_color = SmartUniform::<nglm::Vec3>::new("u_ambientColor", context.clone(), program.clone());
        let mut light_color = SmartUniform::<nglm::Vec3>::new("u_lightColor", context.clone(), program.clone());

        ambient_strength.smart_write(0.3);
        specular_strength.smart_write(0.5);
        ambient_color.smart_write(nglm::vec3(0.8, 0.8, 1.0));
        light_color.smart_write(nglm::vec3(1.0, 1.0, 1.0));

        Self {
            ambient_strength,
            specular_strength,
            ambient_color,
            light_color,
            light_position: SmartUniform::<nglm::Vec3>::new("u_lightPosition", context.clone(), program.clone()),
            camera_position: SmartUniform::<nglm::Vec3>::new("u_cameraPosition", context.clone(), program.clone()),
        }
    }
}
