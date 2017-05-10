#![allow(dead_code)]

use gfx;

pub use gfx_app::{ColorFormat, DepthFormat};

pub type LayerFormat = [f32; 4];
pub type PbrTex = [f32; 4];
pub type VertexSlice<R, V> = (gfx::handle::Buffer<R, V>, gfx::Slice<R>);

gfx_defines!{
    #[derive(PartialEq)]
    vertex CtrPoint {
        pos: [f32; 3] = "a_pos",
    }

    #[derive(PartialEq)]
    vertex V {
        a_pos: [f32; 3] = "a_pos",
    }

    #[derive(PartialEq)]
    vertex Vn {
        a_pos: [f32; 3] = "a_pos",
        a_nor: [f32; 3] = "a_nor",
    }

    #[derive(PartialEq)]
    vertex Vt {
        a_pos: [f32; 3] = "a_pos",
        a_tex: [f32; 2] = "a_tex",
    }

    #[derive(PartialEq)]
    vertex Vtn {
        a_pos: [f32; 3] = "a_pos",
        a_tex: [f32; 2] = "a_tex",
        a_nor: [f32; 3] = "a_nor",
    }

    #[derive(PartialEq)]
    vertex Vtnt {
        a_pos: [f32; 3] = "a_pos",
        a_tex: [f32; 2] = "a_tex",
        a_nor: [f32; 3] = "a_nor",
        a_tan: [f32; 3] = "a_tan",
        a_btn: [f32; 3] = "a_btn",
    }

    constant TransformBlock {
        model: [[f32; 4]; 4] = "model",
        view: [[f32; 4]; 4] = "view",
        proj: [[f32; 4]; 4] = "proj",
    }

    constant LightBlock {
        matrix: [[f32; 4]; 4] = "light_matrix",
        pos: [f32; 4] = "light_pos",
        color: [f32; 4] = "light_color",
        ambient: [f32; 4] = "ambient",
    }

    constant LiveBlock {
        eye_pos: [f32; 4] = "eye_pos",
        gamma: f32 = "gamma",
        exposure: f32 = "exposure",
        time: f32 = "time",
    }

    pipeline deferred {
        verts: gfx::VertexBuffer<Vtnt> = (),
        transform: gfx::ConstantBuffer<TransformBlock> = "transform",
        normal: gfx::TextureSampler<PbrTex> = "normal_tex",
        layer_a: gfx::RenderTarget<LayerFormat> = "layer_a",
        layer_b: gfx::RenderTarget<LayerFormat> = "layer_b",
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline pbr {
        verts: gfx::VertexBuffer<V> = (),
        live: gfx::ConstantBuffer<LiveBlock> = "live",
        light: gfx::ConstantBuffer<LightBlock> = "light",
        layer_a: gfx::TextureSampler<LayerFormat> = "layer_a",
        layer_b: gfx::TextureSampler<LayerFormat> = "layer_b",
        albedo: gfx::TextureSampler<PbrTex> = "albedo_tex",
        metalness: gfx::TextureSampler<PbrTex> = "metalness_tex",
        roughness: gfx::TextureSampler<PbrTex> = "roughness_tex",
        color: gfx::BlendTarget<LayerFormat> = ("f_color", gfx::state::ColorMask::all(), gfx::preset::blend::ADD),
    }

    pipeline ldr {
        verts: gfx::VertexBuffer<V> = (),
        live: gfx::ConstantBuffer<LiveBlock> = "live",
        value: gfx::TextureSampler<LayerFormat> = "value",
        color: gfx::RenderTarget<ColorFormat> = "f_color",
    }
}