#![feature(type_ascription)]
#![feature(box_syntax, box_patterns)]
#![feature(log_syntax, trace_macros)]
#![allow(dead_code)]

#[macro_use]
extern crate glium; // Safe (Rust-friendly) OpenGL + GLFW wrapper 
                    // (not same syntax as c/c++ bindings, but still corresponds 1-to-1)

extern crate nalgebra; // Math library, like glm

mod shaders;
mod camera;
mod project;
mod wavefront;

// use safe GLFW wrapper stuff
use glium::glutin::{self, Event, WindowBuilder, GlProfile, GlRequest};
use glium::{Api, Version, DisplayBuild};
use glium::backend::glutin_backend::GlutinFacade;

use project::Project;

fn main() {
     // create OpenGL context
    let display = WindowBuilder::new()
        .with_dimensions(512, 512)
        .with_title(format!("Sideline"))
        .with_gl_profile(GlProfile::Core) // core profile
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 1))) // as new as possible
        .with_depth_buffer(24)
        .build_glium() // do creation context
        .expect("Rare GLFW error occured, sorry :("); // if some weird error occurs, panic

    let version = display.get_opengl_version();

    // print out version
    println!("Version: {}", match version {
        &Version(Api::Gl, major, minor) => format!("OpenGL {}.{}", major, minor),
        &Version(Api::GlEs, major, minor) => format!("OpenGL ES {}.{}", major, minor),
    });

    // check version
    match version {
        &Version(Api::Gl, 4, minor) if minor >= 1 => (),
        &Version(Api::Gl, major, _) if major > 4 => (),
        &Version(Api::GlEs, _, _) => (),
        _ => panic!("OpenGL 4.1, OpenGL ES, or better is required, exiting"),
    }

    let mut project = Project::new(&display, (512, 512));
    main_loop(&mut project, &display);
}

fn main_loop(project: &mut Project, display: &GlutinFacade) {
    loop {
        // get nice OpenGL-encapsulation-state-management-object-thing
        let mut target = display.draw();
        project.draw(&mut target); // DRAW!!!
        target.finish().unwrap(); // cleanup, check for errors

        // do display events
        for ev in display.poll_events() {
            if let Event::Closed = ev {
                return;
            } else {
                project.event(&ev);
            }
        }
    }

}