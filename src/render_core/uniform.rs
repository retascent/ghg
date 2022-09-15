use paste::paste;
use std::fmt::Debug;
use std::marker::PhantomData;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use crate::application::shaders::ShaderContext;

#[allow(unused_imports)]
use crate::utils::prelude::*;

/// This provides a simple wrapper type for writing to uniform values. Uniform<T> provides strongly-
/// typed uniform values and a simple interface for writing the value to the GPU. SmartUniform<T> is
/// an additional layer of "safety", which allows for writing parameters only when they have changed.
/// This helps simplify the process of updating parameters during an animation loop, because you can
/// simply call smart_write every time, and nothing will happen if the parameter hasn't changed.
///
/// Note, however, that these "smart" uniforms are currently independent of one another; creating
/// multiple that point at the same uniform will cause problems. So it's not a complete solution yet.

#[derive(Clone, Debug)]
#[allow(dead_code)] // name isn't used, but it is useful. TODO: Remove it if not debug
pub struct Uniform<T: Debug> {
    name: String,
    context: WebGl2RenderingContext,
    location: Option<WebGlUniformLocation>,
    phantom_value: PhantomData<T>, // Strongly-typed Uniforms are important
}

impl<T: Clone + Debug + PartialEq + UniformValue> Uniform<T> {
    pub fn new(name: &str, shader_context: &ShaderContext) -> Self {
        let location = shader_context.context.get_uniform_location(&shader_context.program, name);
        Self {
            context: shader_context.context.clone(),
            name: name.to_owned(),
            location,
            phantom_value: PhantomData,
        }
    }

    pub fn write_unchecked(&self, t: T) {
        // let name = &self.name;
        // ghg_log!("Writing uniform: {name} -> {t:?}");
        t.write_to_program(&self.context, &self.location);
    }
}

#[derive(Debug)]
pub struct SmartUniform<T: Debug> {
    uniform: Uniform<T>,
    last_value: Option<T>,
}

impl<T: Clone + Debug + PartialEq + UniformValue> SmartUniform<T> {
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
    fn write_to_program(self, context: &WebGl2RenderingContext, location: &Option<WebGlUniformLocation>) where Self: Sized;
}

macro_rules! impl_uniform_creator_fns {
    ($type_name:ty, $short_name:ident) => {
        paste! {
            #[allow(dead_code)] // Used in doc string, but the compiler still complains
            const [< $short_name:upper _STR >]: &str = stringify!($type_name); // TODO: This means this macro has to come first.

            #[allow(dead_code)]
            #[doc = "Creates a new `Uniform<" [< $short_name:upper _STR >] ">`."]
            /// Creates a new Uniform<> of the given type.
            pub fn [< new_ $short_name >](name: &str, shader_context: &ShaderContext) -> Uniform<$type_name> {
                Uniform::new(name, shader_context)
            }

            #[allow(dead_code)]
            #[doc = "Creates and initializes a new `Uniform<" [< $short_name:upper _STR >] ">`."]
            /// Creates and initializes a new Uniform<> of the given type.
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
            #[doc = "Creates a new `SmartUniform<" [< $short_name:upper _STR >] ">`."]
            /// Creates a new SmartUniform<> of the given type.
            pub fn [< new_smart_ $short_name >](name: &str, shader_context: &ShaderContext) -> SmartUniform<$type_name> {
                SmartUniform::new(name, shader_context)
            }

            #[allow(dead_code)]
            #[doc = "Creates and initializes a new `SmartUniform<" [< $short_name:upper _STR >] ">`."]
            /// Creates and initializes a new SmartUniform<> of the given type.
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
            fn write_to_program(self, context: &WebGl2RenderingContext, location: &Option<WebGlUniformLocation>) {
                context.$gl_call(location.as_ref(), self);
            }
        }

        impl_uniform_creator_fns!($type_name, $short_name);
        impl_smart_uniform_creator_fns!($type_name, $short_name);
    };

    // Self is a primitive type, and its own short name; pass self directly to the OpenGL function call
    ($type_name:ident, $gl_call:ident) => {
        impl_uniform!($type_name, $type_name, $gl_call);
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
            fn write_to_program(self, context: &WebGl2RenderingContext, location: &Option<WebGlUniformLocation>) {
                context.$gl_call(location.as_ref(), $( $(self.$field)* $(self.$method())* $($param)* ,)+);
            }
        }

        impl_uniform_creator_fns!($type_name, $short_name);
        impl_smart_uniform_creator_fns!($type_name, $short_name);
    };
}


impl_uniform!(i32, uniform1i);
impl_uniform!(f32, uniform1f);
impl_uniform!(nglm::Vec3, vec3, uniform3f, self.x, self.y, self.z);
impl_uniform!(nglm::Vec4, vec4, uniform4f, self.x, self.y, self.z, self.w);
impl_uniform!(nglm::Mat4, mat4, uniform_matrix4fv_with_f32_array, just false, call self.as_slice());

// TODO: Way more implementations
