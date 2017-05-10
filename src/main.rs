#![feature(slice_patterns)]

#[macro_use]
extern crate gfx; // Safe (Rust-friendly) graphics
extern crate gfx_app;
extern crate winit;

extern crate cgmath; // Math library, like glm

extern crate image;

mod shaders; // shaders.rs
mod camera; // camera.rs
mod ctrpts;
mod define;
mod app;
mod wavefront;

pub const DEFAULT_SIZE: (u32, u32) = (512, 512);

fn main() {
    let wb = winit::WindowBuilder::new()
        .with_title("PBR with gfx-rs")
        .with_dimensions(DEFAULT_SIZE.0, DEFAULT_SIZE.1);
    gfx_app::launch_gl3::<app::App<_, _>>(wb);
}