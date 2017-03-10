#[macro_use]
mod util;
use self::util::BuildShader;

use glium::uniforms::{Uniforms, UniformValue};
use glium::backend::{Facade};

use nalgebra::{Point3, Matrix4, Eye};

const LIGHT_COUNT: usize = 2;

shader!(phong {
    vertex_shader: file("shaders/simple_transform.v.glsl")
        .define("VIEWPROJ")
        .define("NORM"),
    fragment_shader: file("shaders/wire_phong.f.glsl")
        .define_to("LIGHT_COUNT", LIGHT_COUNT)
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm")
});

shader!(wire_phong {
    vertex_shader: file("shaders/simple_transform.v.glsl")
        .define("VIEWPROJ")
        .define("NORM"),
    geometry_shader: file("shaders/wire.g.glsl")
        .define_to("I_POS", "v_pos")
        .define_to("I_NORM", "v_norm"),
    fragment_shader: file("shaders/phong.f.glsl")
        .define_to("LIGHT_COUNT", LIGHT_COUNT)
        .define("WIRE")
        .define_to("I_POS", "g_pos")
        .define_to("I_NORM", "g_norm")
});

#[derive(Debug, Clone, Copy)]
pub struct PhongLight {
    pub pos: Point3<f32>,
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct PhongUniforms {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub cam_pos: Point3<f32>,
    pub material_spec: [f32; 3],
    pub material_diff: [f32; 3],
    pub material_hard: f32,
    pub lights: [PhongLight; LIGHT_COUNT],
    pub ambient: [f32; 3],
    pub line_color: [f32; 3],
    pub line_width: f32,
    pub screen_size: (u32, u32),
    pub show_wireframe: bool,
}

impl Default for PhongUniforms {
    fn default() -> PhongUniforms {
        PhongUniforms {
            model: Matrix4::new_identity(4),
            view: Matrix4::new_identity(4),
            proj:  Matrix4::new_identity(4),
            cam_pos: Point3::new(0., 0., 0.),
            material_spec: [0.; 3],
            material_diff: [0.; 3],
            material_hard: 0.,
            lights: [PhongLight { pos: Point3::new(0., 0., 0.), color: [0., 0., 0.] }; LIGHT_COUNT],
            ambient: [0.; 3],
            line_color: [0.; 3],
            line_width: 0.,
            screen_size: (0, 0),
            show_wireframe: false,
        }
    }
}

impl Uniforms for PhongUniforms {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        use self::UniformValue::*;

        f("model", Mat4(*self.model.as_ref()));
        f("view", Mat4(*self.view.as_ref()));
        f("proj", Mat4(*self.proj.as_ref()));

        f("cam_pos", Vec3(*self.cam_pos.as_ref()));

        f("material_spec", Vec3(self.material_spec));
        f("material_diff", Vec3(self.material_diff));
        f("material_hard", Float(self.material_hard));

        for i in 0..self.lights.len() {
            f(&format!("lights[{}].pos", i), Vec3(*self.lights[i].pos.as_ref()));
            f(&format!("lights[{}].color", i), Vec3(self.lights[i].color));
        }

        f("ambient", Vec3(self.ambient));

        f("line_color", Vec3(self.line_color));
        f("line_width", Float(self.line_width));

        f("screen_width", SignedInt(self.screen_size.0 as i32));
        f("screen_height", SignedInt(self.screen_size.1 as i32));

        f("show_wireframe", SignedInt(if self.show_wireframe { 1 } else { 0 }));
    }
}