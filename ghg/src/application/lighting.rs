use crate::application::shaders::ShaderContext;
use crate::render_core::uniform;
use crate::render_core::uniform::SmartUniform;

pub struct LightParameters {
	pub ambient_strength: SmartUniform<f32>,
	pub specular_strength: SmartUniform<f32>,
	pub ambient_color: SmartUniform<nglm::Vec3>,
	pub light_color: SmartUniform<nglm::Vec3>,
	pub light_position: SmartUniform<nglm::Vec3>,
	pub camera_position: SmartUniform<nglm::Vec3>,
}

impl LightParameters {
	pub fn new(shader_context: &ShaderContext) -> Self {
		let ambient_strength = uniform::init_smart_f32("u_ambientStrength", &shader_context, 0.3);
		let specular_strength = uniform::init_smart_f32("u_specularStrength", &shader_context, 0.5);
		let ambient_color =
			uniform::init_smart_vec3("u_ambientColor", &shader_context, nglm::vec3(0.8, 0.8, 1.0));
		let light_color =
			uniform::init_smart_vec3("u_lightColor", &shader_context, nglm::vec3(1.0, 1.0, 1.0));

		Self {
			ambient_strength,
			specular_strength,
			ambient_color,
			light_color,
			light_position: uniform::new_smart_vec3("u_lightPosition", &shader_context),
			camera_position: uniform::new_smart_vec3("u_cameraPosition", &shader_context),
		}
	}
}
