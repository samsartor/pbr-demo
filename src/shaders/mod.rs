#![allow(dead_code)]

#[macro_use]
mod util;

use gfx;
use self::util::file;

pub const LIGHT_COUNT: usize = 2;

shader!(deferred {
            vertex: file("shaders/transform.v.glsl")
                .define("VIEWPROJ")
                .define("TAN")
                .define("TEX")
                .define("NORM"),
            fragment: file("shaders/deferred.f.glsl")
        });

shader!(shadow {
            vertex: file("shaders/transform.v.glsl")
                .define("VIEWPROJ"),
            fragment: file("shaders/shadow.f.glsl")
        });

shader!(pbr {
            vertex: file("shaders/blit.v.glsl"),
            fragment: file("shaders/pbr.f.glsl")
        });