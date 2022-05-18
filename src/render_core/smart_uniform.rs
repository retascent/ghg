/// This provides a wrapper type for simplifying the uniform write process. By using a SmartUniform,
/// you get not only encapsulation of the uniform destination, but also ensure that a uniform is not
/// written multiple times if it already has the value.
///
/// This helps simplify the process of updating parameters during an animation loop, because you can
/// simply call smart_write ever time, and nothing will happen if the parameter hasn't changed.

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

#[derive(Clone, Debug)]
#[allow(dead_code)] // For debug, basically
pub struct SmartUniform<T> {
    context: WebGl2RenderingContext,
    name: String,
    location: WebGlUniformLocation,
    last_value: Option<T>,
}

impl<T: PartialEq + Clone + UniformValue> SmartUniform<T> {
    pub fn new(name: String, context: WebGl2RenderingContext, program: WebGlProgram) -> Self {
        let location = context.get_uniform_location(&program, name.as_str()).unwrap();
        Self {
            context,
            name,
            location,
            last_value: None,
        }
    }

    pub fn smart_write(&mut self, t: T) {
        if self.last_value.is_none() || &t != self.last_value.as_ref().unwrap() {
            t.clone().write_to_program(&self.context, &self.location);
            self.last_value = Some(t);
        }
    }
}

pub trait UniformValue {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) where Self: Sized;
}

impl UniformValue for i32 {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
        context.uniform1i(Some(location), self);
    }
}

impl UniformValue for f32 {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
        context.uniform1f(Some(location), self);
    }
}

impl UniformValue for nglm::Vec3 {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
        context.uniform3f(Some(location), self.x, self.y, self.z);
    }
}

impl UniformValue for nglm::Vec4 {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
        context.uniform4f(Some(location), self.x, self.y, self.z, self.w);
    }
}

impl UniformValue for nglm::Mat4 {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
        context.uniform_matrix4fv_with_f32_array(Some(location), false, self.as_slice())
    }
}

// TODO: Way more implementations
