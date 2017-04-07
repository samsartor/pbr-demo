use glium::glutin::{Event, MouseButton, VirtualKeyCode, MouseScrollDelta, ElementState};
use glium::{Program, VertexBuffer, IndexBuffer, Surface, DrawParameters};
use glium::texture::{Texture2d, SrgbTexture2d};
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::GlutinFacade;
use glium::uniforms::{Sampler, MinifySamplerFilter, MagnifySamplerFilter};
use glium;

use nalgebra::{Point3, Vector3};

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::rc::Rc;
use std::clone::Clone;
use std::fs::File;
use std::io::BufReader;

use wavefront;
use image;
use shaders::{self, GbugffUniforms};
use camera::{Camera, BasicPerspCamera, new_perspective};
use gbuff::Gbuff;

const PI: f32 = ::std::f32::consts::PI;
const HPI: f32 = 0.5 *  PI;

#[derive(Copy,Clone)]
enum ViewMode {
    PBR,
    DEBUG,
    PHONG,
}

impl ViewMode {
    fn cycle(self) -> ViewMode {
        use self::ViewMode::*;
        match self {
            PBR => DEBUG,
            DEBUG => PHONG,
            PHONG => PBR,
        }
    }
}

fn load_image_rgb<'a>(display: &'a GlutinFacade, path: &str) -> Texture2d {
    let image = image::load(BufReader::new(File::open(path).expect("Image \"{}\" not found")), image::PNG).unwrap().to_rgb();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgb_reversed(image.into_raw(), image_dimensions);
    Texture2d::new(display, image).unwrap()
}

fn load_image_srgb<'a>(display: &'a GlutinFacade, path: &str) -> SrgbTexture2d {
    let image = image::load(BufReader::new(File::open(path).expect("Image \"{}\" not found")), image::PNG).unwrap().to_rgb();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgb_reversed(image.into_raw(), image_dimensions);
    SrgbTexture2d::new(display, image).unwrap()
}

pub struct PbrTextures<T, S> {
    albedo: S,
    metalness: T,
    roughness: T,
    normal: T,
}

pub struct Project {
    // reference back to the gl context
    display: Rc<GlutinFacade>,
    // when was the program Sized
    start: Instant,
    // when was draw last called
    last_draw: Option<Instant>,
    // the display size
    screen_size: (u32, u32),
    // last mouse position (if on screen)
    mouse_pos: Option<(f32, f32)>,
    // are keys down
    keys: HashMap<VirtualKeyCode, bool>,
    // are mouse buttons down, and at what position where they first clicked?
    mouse: HashMap<MouseButton, Option<(f32, f32)>>,
    // teapot mesh
    mesh: (VertexBuffer<wavefront::Vtnt>, IndexBuffer<u32>),
    // full screen quad
    fsquad: (VertexBuffer<wavefront::V>, IndexBuffer<u8>),
    // shaders
    gbuff_render: Program, // store deferred gbuff data
    gbuff_view: Program,
    pbr: Program,
    phong: Program,
    // pbr textures
    pbrtex: PbrTextures<Texture2d, SrgbTexture2d>,
    // shader mode
    shade_mode: ViewMode,
    // deferred data
    gbuff: Gbuff,
    // arcball theta, phi, radius
    arcball: (f32, f32, f32),
    camera: Option<BasicPerspCamera>,
    exposure: f32,
    gamma: f32,
    norm_map_strength: f32,
}

impl Project {
    pub fn new(display: Rc<GlutinFacade>, start_size: (u32, u32), model: &str, tex_folder: &str) -> Project {
        let model = wavefront::load_from_path(model).unwrap();

        let display_ref = display.as_ref();

        let fsquad = (
            VertexBuffer::immutable(display.as_ref(), &[
                wavefront::V { a_pos: [ -1., -1., 0. ] },
                wavefront::V { a_pos: [ -1.,  1., 0. ] },
                wavefront::V { a_pos: [  1., -1., 0. ] },
                wavefront::V { a_pos: [  1.,  1., 0. ] },
            ]).unwrap(), 
            IndexBuffer::immutable(display_ref, PrimitiveType::TrianglesList, &[0, 1, 2, 2, 1, 3]).unwrap()
        );

        Project { 
            display: display.clone(),
            start: Instant::now(),
            last_draw: None,
            screen_size: start_size,
            mouse_pos: None,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            mesh: (model.vertex_buffer(display_ref).unwrap(), model.index_buffer(display_ref, PrimitiveType::TrianglesList).unwrap()),
            fsquad: fsquad,
            gbuff_render: shaders::gbuff(display_ref).expect("gbuff program error"),
            gbuff_view: shaders::gbuff_view(display_ref).expect("gbuff_view program error"),
            pbr: shaders::pbr(display_ref).expect("pbr program error"),
            phong: shaders::phong(display_ref).expect("phong program error"),
            shade_mode: ViewMode::DEBUG,
            gbuff: Gbuff::new(display.clone(), start_size),
            pbrtex: PbrTextures {
                albedo: load_image_srgb(display_ref, &(tex_folder.to_owned() + "/albedo.png")),
                metalness: load_image_rgb(display_ref, &(tex_folder.to_owned() + "/metalness.png")),
                roughness: load_image_rgb(display_ref, &(tex_folder.to_owned() + "/roughness.png")),
                normal: load_image_rgb(display_ref, &(tex_folder.to_owned() + "/normal.png")),
            },
            arcball: (0., 0., 10.),
            camera: None,
            exposure: 1.0,
            gamma: 2.2,
            norm_map_strength: 0.4,
        }
    }

    pub fn event(&mut self, event: &Event) {
        use Event::*;
        match *event {
            KeyboardInput(state, _, Some(code)) => {
                use self::ElementState::*;

                match state {
                    Pressed => self.keys.insert(code, true),
                    Released => self.keys.insert(code, false),
                };

                match (state, code) {
                    (Pressed, VirtualKeyCode::M) => self.shade_mode = self.shade_mode.cycle(),
                    (Pressed, VirtualKeyCode::Up) => self.exposure *= 1.2,
                    (Pressed, VirtualKeyCode::Down) => self.exposure /= 1.2,
                    (Pressed, VirtualKeyCode::Right) => self.gamma += 0.1,
                    (Pressed, VirtualKeyCode::Left) => { self.gamma -= 0.1; if self.gamma < 0.0 { self.gamma = 0.0 } },
                    (Pressed, VirtualKeyCode::RBracket) => self.norm_map_strength += 0.05,
                    (Pressed, VirtualKeyCode::LBracket) => self.norm_map_strength -= 0.05,
                    _ => (),
                };
                // TODO on-press events
            },
            Resized(w, h) => {
                self.screen_size = (w, h);
                self.gbuff = Gbuff::new(self.display.clone(), self.screen_size);
            },
            MouseMoved(x, y) => {
                let (w, h) = self.screen_size;
                if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                    self.mouse_pos = None;
                } else {
                    self.mouse_pos = Some((
                        x as f32 / w as f32 * 2.0 - 1.0,
                        y as f32 / h as f32 * 2.0 - 1.0
                    ))
                }
            }, 
            MouseLeft => self.mouse_pos = None,
            MouseInput(state, butt) => {
                use self::ElementState::*;

                match state {
                    Pressed => self.mouse.insert(butt, self.mouse_pos),
                    Released => self.mouse.insert(butt, None),
                };

                // TODO
            }
            MouseWheel(MouseScrollDelta::LineDelta(_, y), _) => {
                self.arcball.2 *= 1.05f32.powf(y);
            },
            _ => (),
        }
    }

    pub fn key_down(&self, code: VirtualKeyCode) -> bool {
        self.keys.get(&code).cloned().unwrap_or(false)
    }

    pub fn mouse_down(&self, butt: MouseButton) -> bool {
        self.mouse.get(&butt).cloned().unwrap_or(None).is_some()
    }

    pub fn mouse_drag(&self, butt: MouseButton) -> Option<(f32, f32)> {
        match (self.mouse.get(&butt), self.mouse_pos) {
            (Some(&Some((x0, y0))), Some((x1, y1))) => Some((x1 - x0, y1 - y0)),
            _ => None,
        }
    }

    pub fn reset_mouse_drag(&mut self, butt: MouseButton) -> Option<(f32, f32)> {
        match (self.mouse.get_mut(&butt), self.mouse_pos) {
            (Some(&mut Some((ref mut x0, ref mut y0))), Some((x1, y1))) => {
                let out = Some((x1 - *x0, y1 - *y0));
                *x0 = x1;
                *y0 = y1;
                out
            },
            _ => None,
        }
    }

    fn update_draw_timer(&mut self) -> (f64, f64) {
        let now = Instant::now();
        self.last_draw = Some(now);

        let elapsed = now - self.start;
        let delta = now - self.last_draw.unwrap_or(now);

        (duration_to_secs(&elapsed), duration_to_secs(&delta))
    }

    fn get_camera(&self) -> BasicPerspCamera {
        let arcpos = {
            let cosphi = self.arcball.1.cos();
            let x = self.arcball.0.cos() * cosphi * self.arcball.2;
            let z = self.arcball.0.sin() * cosphi * self.arcball.2;
            let y = self.arcball.1.sin() * self.arcball.2;
            Point3::new(x, y, z)
        };

        new_perspective(
            arcpos, 
            Point3::new(0., 0., 0.),
            Vector3::new(0., 1., 0.),
            self.screen_size.0 as f32 / self.screen_size.1 as f32,
            25., 0.1, 1000.)
    }

    pub fn draw<S: Surface>(&mut self, surf: &mut S) {
        let (elapsed, delta) = self.update_draw_timer();

        if let Some((dx, dy)) = self.reset_mouse_drag(MouseButton::Left) {
            self.arcball.0 += dx * 1.2;
            self.arcball.1 += dy * 1.2;

            if self.arcball.1 < -HPI + 0.01 { self.arcball.1 = -HPI + 0.01; }
            if self.arcball.1 >  HPI - 0.01 { self.arcball.1 =  HPI - 0.01; }
        }

        let camera = self.get_camera();

        //=================//
        //                 //
        // Geometry Pass   //
        //                 //
        //=================//

        {
            let draw = self.gbuff.get_mut_fbo();
            draw.clear_depth(1.0);
            draw.clear_color(0.0, 0.0, 0.0, 0.0);

            let params = DrawParameters {
                depth: glium::Depth {
                    test: glium::DepthTest::IfLess,
                    write: true,
                    .. Default::default()
                },
                .. Default::default()
            };

            draw.draw(&self.mesh.0, &self.mesh.1, &self.gbuff_render, &GbugffUniforms {
                view: camera.get_view(),
                proj: camera.get_proj(),
                norm_map_strength: self.norm_map_strength,
                ..Default::default()
            }, &params).unwrap();
        }

        //=================//
        //                 //
        // Deferred Pass   //
        //                 //
        //=================//

        let layera = Sampler::new(self.gbuff.get_layera());
        layera.minify_filter(MinifySamplerFilter::Nearest);
        layera.magnify_filter(MagnifySamplerFilter::Nearest);

        let layerb = Sampler::new(self.gbuff.get_layerb());
        layerb.minify_filter(MinifySamplerFilter::Nearest);
        layerb.magnify_filter(MagnifySamplerFilter::Nearest);

        use self::ViewMode::*;
        match self.shade_mode {
            DEBUG => surf.draw(&self.fsquad.0, &self.fsquad.1, &self.gbuff_view, &uniform!(
                layera: layera,
                layerb: layerb,
                pos_range: (0.0f32, 0.333f32),
                tex_range: (0.333f32, 0.666f32),
                norm_range: (0.666f32, 1.0f32),
                albedo_tex: &self.pbrtex.albedo,
                metalness_tex: &self.pbrtex.metalness,
            ), &Default::default()).unwrap(),
            PBR => surf.draw(&self.fsquad.0, &self.fsquad.1, &self.pbr, &uniform!(
                layera: layera,
                layerb: layerb,
                camera_pos: *camera.eye.as_ref(),
                albedo_tex: &self.pbrtex.albedo,
                roughness_tex: &self.pbrtex.roughness,
                metalness_tex: &self.pbrtex.metalness,
                normal_tex: &self.pbrtex.normal,
                gamma: self.gamma,
                exposure: self.exposure,
                time: elapsed as f32,
            ), &Default::default()).unwrap(),
            PHONG => surf.draw(&self.fsquad.0, &self.fsquad.1, &self.phong, &uniform!(
                layera: layera,
                layerb: layerb,
                camera_pos: *camera.eye.as_ref(),
                albedo_tex: &self.pbrtex.albedo,
                roughness_tex: &self.pbrtex.roughness,
                time: elapsed as f32,
            ), &Default::default()).unwrap(),
        }        
    }   
}

fn duration_to_secs(dur: &Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 * 1e-9f64
}