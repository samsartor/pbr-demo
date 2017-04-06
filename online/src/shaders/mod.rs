#[macro_use]
mod util;
use self::util::BuildShader;

use glium::uniforms::{Uniforms, UniformValue};
use glium::backend::{Facade};
use glium::texture::Texture2d;

use nalgebra::{Point3, Matrix4, Eye};

const LIGHT_COUNT: usize = 2;

shader!(gbuff {
    vertex_shader: file("shaders/simple_transform.v.glsl")
        .define("NORM")
        .define("TEX")
        .define("VIEWPROJ"),
    fragment_shader: file("shaders/gbuff.f.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm")
        .define_to("I_TEX", "v_tex")
});

shader!(gbuff_view {
    vertex_shader: file("shaders/blit.v.glsl"),
    fragment_shader: file("shaders/gbuff_view.f.glsl")
});

shader!(pbr {
    vertex_shader: file("shaders/blit.v.glsl"),
    fragment_shader: file("shaders/pbr.f.glsl")
});

shader!(phong {
    vertex_shader: file("shaders/blit.v.glsl"),
    fragment_shader: file("shaders/deferred_phong.f.glsl")
});

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub pos: Point3<f32>,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct GbugffUniforms {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

impl Default for GbugffUniforms {
    fn default() -> GbugffUniforms {
        GbugffUniforms {
            model: Matrix4::new_identity(4),
            view: Matrix4::new_identity(4),
            proj:  Matrix4::new_identity(4),
        }
    }
}

impl Uniforms for GbugffUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        use self::UniformValue::*;

        f("model", Mat4(*self.model.as_ref()));
        f("view", Mat4(*self.view.as_ref()));
        f("proj", Mat4(*self.proj.as_ref()));
    }
}