// #![feature(concat_idents)]

use std::marker::PhantomData;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

/// This provides a simple wrapper type for writing to uniform values. Uniform<T> provides strongly-
/// typed uniform values and a simple interface for writing the value to the GPU. SmartUniform<T> is
/// an additional layer of "safety", which allows for writing parameters only when they have changed.
/// This helps simplify the process of updating parameters during an animation loop, because you can
/// simply call smart_write every time, and nothing will happen if the parameter hasn't changed.
///
/// Note, however, that these "smart" uniforms are currently independent of one another; creating
/// multiple that point at the same uniform will cause problems. So it's not a complete solution yet.

#[derive(Clone, Debug)]
pub struct ShaderContext {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
}

impl ShaderContext {
    pub fn new(context: &WebGl2RenderingContext, program: &WebGlProgram) -> Self {
        Self {
            context: context.clone(),
            program: program.clone(),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // name isn't used, but it is useful. TODO: Remove it if not debug
pub struct Uniform<T> {
    name: String,
    context: WebGl2RenderingContext,
    location: WebGlUniformLocation,
    phantom_value: PhantomData<T>, // Strongly-typed Uniforms are important
}

impl<T: PartialEq + Clone + UniformValue> Uniform<T> {
    pub fn new(name: &str, shader_context: &ShaderContext) -> Self {
        let location = shader_context.context.get_uniform_location(&shader_context.program, name)
            .expect(format!("Failed to find uniform {name}").as_str());
        Self {
            context: shader_context.context.clone(),
            name: name.to_owned(),
            location,
            phantom_value: PhantomData,
        }
    }

    pub fn write_unchecked(&self, t: T) {
        t.write_to_program(&self.context, &self.location);
    }
}

#[derive(Debug)]
pub struct SmartUniform<T> {
    uniform: Uniform<T>,
    last_value: Option<T>,
}

impl<T: PartialEq + Clone + UniformValue> SmartUniform<T> {
    pub fn new(name: &str, shader_context: &ShaderContext) -> Self {
        Self {
            uniform: Uniform::new(name, shader_context),
            last_value: None,
        }
    }

    // pub fn get_unchecked(&self) -> Uniform<T> {
    //     self.uniform.clone()
    // }

    pub fn smart_write(&mut self, t: T) {
        if self.last_value.is_none() || &t != self.last_value.as_ref().unwrap() {
            self.uniform.write_unchecked(t.clone());
            self.last_value = Some(t);
        }
    }
}

pub trait UniformValue {
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) where Self: Sized;
}

use paste::paste;

macro_rules! impl_uniform_creator_fns {
    ($type_name:ty, $short_name:ident) => {
        paste! {
            #[allow(dead_code)]
            // #[doc = "Creates a new `Uniform<" $type_name ">`."]
            pub fn [< new_ $short_name >](name: &str, shader_context: &ShaderContext) -> Uniform<$type_name> {
                Uniform::new(name, shader_context)
            }

            #[allow(dead_code)]
            // #[doc = "Creates and initializes a new `Uniform<" $type_name ">`."]
            pub fn [< init_ $short_name >](name: &str, shader_context: &ShaderContext, value: $type_name) -> Uniform<$type_name> {
                let u = Uniform::new(name, shader_context);
                u.write_unchecked(value);
                u
            }
        }
    };
}

macro_rules! impl_smart_uniform_creator_fns {
    ($type_name:ty, $short_name:ident) => {
        paste! {
            #[allow(dead_code)]
            // #[doc = "Creates a new `SmartUniform<" $type_name ">`."]
            pub fn [< new_smart_ $short_name >](name: &str, shader_context: &ShaderContext) -> SmartUniform<$type_name> {
                SmartUniform::new(name, shader_context)
            }

            #[allow(dead_code)]
            // #[doc = "Creates and initializes a new `SmartUniform<" $type_name ">`."]
            pub fn [< init_smart_ $short_name >](name: &str, shader_context: &ShaderContext, value: $type_name) -> SmartUniform<$type_name> {
                let mut u = SmartUniform::new(name, shader_context);
                u.smart_write(value);
                u
            }
        }
    };
}

macro_rules! impl_uniform {
    // Self is a primitive type; pass self directly to the OpenGL function call
    ($type_name:ty, $short_name:ident, $gl_call:ident) => {
        impl UniformValue for $type_name {
            fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
                context.$gl_call(Some(location), self);
            }
        }

        impl_uniform_creator_fns!($type_name, $short_name);
        impl_smart_uniform_creator_fns!($type_name, $short_name);
    };

    // Ugly form that takes parameters. You can pass these things:
    //    - self.some_field
    //    - call self.some_method()
    //    - just some_expression
    // Definitely incomplete and hacky, but I'm learning macros.
    (
        $type_name:ty, $short_name:ident, $gl_call:ident,
        $( $(self.$field:ident)? $(call self.$method:ident())? $(just $param:expr)? ),+
    ) => {
        impl UniformValue for $type_name {
            fn write_to_program(self, context: &WebGl2RenderingContext, location: &WebGlUniformLocation) {
                context.$gl_call(Some(location), $( $(self.$field)* $(self.$method())* $($param)* ,)+);
            }
        }

        impl_uniform_creator_fns!($type_name, $short_name);
        impl_smart_uniform_creator_fns!($type_name, $short_name);
    };
}


impl_uniform!(i32, i32, uniform1i);
impl_uniform!(f32, f32, uniform1f);
impl_uniform!(nglm::Vec3, vec3, uniform3f, self.x, self.y, self.z);
impl_uniform!(nglm::Vec4, vec4, uniform4f, self.x, self.y, self.z, self.w);
impl_uniform!(nglm::Mat4, mat4, uniform_matrix4fv_with_f32_array, just false, call self.as_slice());

// TODO: Way more implementations
