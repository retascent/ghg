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
        Self {
            ambient_strength: SmartUniform::<f32>::new("u_ambientStrength".to_owned(), context.clone(), program.clone()),
            specular_strength: SmartUniform::<f32>::new("u_specularStrength".to_owned(), context.clone(), program.clone()),
            ambient_color: SmartUniform::<nglm::Vec3>::new("u_ambientColor".to_owned(), context.clone(), program.clone()),
            light_color: SmartUniform::<nglm::Vec3>::new("u_lightColor".to_owned(), context.clone(), program.clone()),
            light_position: SmartUniform::<nglm::Vec3>::new("u_lightPosition".to_owned(), context.clone(), program.clone()),
            camera_position: SmartUniform::<nglm::Vec3>::new("u_cameraPosition".to_owned(), context.clone(), program.clone()),
        }
    }
}
