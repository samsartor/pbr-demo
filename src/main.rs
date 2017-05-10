#![feature(slice_patterns)]

#[macro_use]
extern crate gfx; // Safe (Rust-friendly) graphics
extern crate gfx_app; // easy main loop/other window stuff
extern crate winit; // windowing library, built on/part of glutin (equiv. to GLFW)

extern crate cgmath; // Math library, like glm

extern crate image; // image loading
#[macro_use]
extern crate clap; // command-line args
extern crate rand; // random number gen

mod shaders; // shaders.rs
mod camera; // camera.rs
mod define;
mod app;
mod wavefront;

pub const DEFAULT_SIZE: (u32, u32) = (1024, 1024);

fn main() {
    let wb = winit::WindowBuilder::new()
        .with_title("PBR with gfx-rs")
        .with_dimensions(DEFAULT_SIZE.0, DEFAULT_SIZE.1);
    gfx_app::launch_gl3::<app::App<_, _>>(wb);
}