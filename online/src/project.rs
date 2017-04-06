use glium::glutin::{Event, MouseButton, VirtualKeyCode, MouseScrollDelta, ElementState};
use glium::{Program, VertexBuffer, IndexBuffer, Surface, DrawParameters};
use glium::framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer};
use glium::texture::{DepthFormat, UncompressedFloatFormat, MipmapsOption, Texture2d, SrgbTexture2d};
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::GlutinFacade;
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

pub struct FboStore {
    depth: Rc<DepthRenderBuffer>,
    layera: Rc<Texture2d>,
    layerb: Rc<Texture2d>,
}

impl FboStore {
    pub fn init_fbo<'a>(&'a self, display: &GlutinFacade) -> MultiOutputFrameBuffer<'a> {
        MultiOutputFrameBuffer::with_depth_buffer(
            display, 
            vec![("layera", &*self.layera), ("layerb", &*self.layerb)], 
            &*self.depth
        ).unwrap()
    }
}

fn load_image_rgb<'a>(display: &'a GlutinFacade, path: &str) -> SrgbTexture2d {
    let image = image::load(BufReader::new(File::open(path).expect("Image \"{}\" not found")), image::PNG).unwrap().to_rgb();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgb_reversed(image.into_raw(), image_dimensions);
    SrgbTexture2d::new(display, image).unwrap()
}

pub struct PbrTextures<T> {
    albedo: T,
    metalness: T,
    roughness: T,
    normal: T,
}

pub struct Project<'a> {
    // reference back to the gl context
    display: &'a GlutinFacade,
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
    mesh: (VertexBuffer<wavefront::Vtn>, IndexBuffer<u32>),
    // full screen quad
    fsquad: (VertexBuffer<wavefront::V>, IndexBuffer<u8>),
    // shaders
    gbuff: Program, // store deferred gbuff data
    gbuff_view: Program,
    pbr: Program,
    phong: Program,
    // pbr textures
    pbrtex: PbrTextures<SrgbTexture2d>,
    // shader mode
    shade_mode: ViewMode,
    // deferred data
    depth: Rc<DepthRenderBuffer>,
    layera: Rc<Texture2d>,
    layerb: Rc<Texture2d>,
    // arcball theta, phi, radius
    arcball: (f32, f32, f32),
    camera: Option<BasicPerspCamera>,

}

impl<'a> Project<'a> {
    pub fn new(display: &'a GlutinFacade, start_size: (u32, u32)) -> Project<'a> {
        let sphere = wavefront::load_from_path("sphere.obj").unwrap();

        let layera = Texture2d::empty_with_format(
            display, 
            UncompressedFloatFormat::F32F32F32F32,
            MipmapsOption::NoMipmap,
            start_size.0, start_size.1).unwrap();

        let layerb = Texture2d::empty_with_format(
            display, 
            UncompressedFloatFormat::F32F32F32F32,
            MipmapsOption::NoMipmap,
            start_size.0, start_size.1).unwrap();

        let depth = DepthRenderBuffer::new(display, DepthFormat::I24, start_size.0, start_size.1).unwrap();

        let fsquad = (
            VertexBuffer::immutable(display, &[
                wavefront::V { a_pos: [ -1., -1., 0. ] },
                wavefront::V { a_pos: [ -1.,  1., 0. ] },
                wavefront::V { a_pos: [  1., -1., 0. ] },
                wavefront::V { a_pos: [  1.,  1., 0. ] },
            ]).unwrap(), 
            IndexBuffer::immutable(display, PrimitiveType::TrianglesList, &[0, 1, 2, 2, 1, 3]).unwrap()
        );

        Project { 
            display: display,
            start: Instant::now(),
            last_draw: None,
            screen_size: start_size,
            mouse_pos: None,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            mesh: (sphere.vertex_buffer(display).unwrap(), sphere.index_buffer(display, PrimitiveType::TrianglesList).unwrap()),
            fsquad: fsquad,
            gbuff: shaders::gbuff(display).expect("gbuff program error"),
            gbuff_view: shaders::gbuff_view(display).expect("gbuff_view program error"),
            pbr: shaders::pbr(display).expect("pbr program error"),
            phong: shaders::phong(display).expect("phong program error"),
            shade_mode: ViewMode::DEBUG,
            depth: Rc::new(depth),
            layera: Rc::new(layera),
            layerb: Rc::new(layerb),
            pbrtex: PbrTextures {
                albedo: load_image_rgb(display, "textures/albedo.png"),
                metalness: load_image_rgb(display, "textures/metalness.png"),
                roughness: load_image_rgb(display, "textures/roughness.png"),
                normal: load_image_rgb(display, "textures/normal.png"),
            },
            arcball: (0., 0., 10.),
            camera: None,
        }
    }

    pub fn get_store(&self) -> FboStore {
        FboStore {
            depth: self.depth.clone(),
            layera: self.layera.clone(),
            layerb: self.layerb.clone(),
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
                    _ => (),
                };
                // TODO on-press events
            },
            Resized(w, h) => {
                self.screen_size = (w, h)

                // TODO resize FBOs
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

    pub fn post<S: Surface>(&mut self, draw: &mut S) {
        let camera = self.get_camera();

        use self::ViewMode::*;
        match self.shade_mode {
            DEBUG => draw.draw(&self.fsquad.0, &self.fsquad.1, &self.gbuff_view, &uniform!(
                layera: self.layera.as_ref(),
                layerb: self.layerb.as_ref(),
                pos_range: (0.0f32, 0.333f32),
                tex_range: (0.333f32, 0.666f32),
                norm_range: (0.666f32, 1.0f32),
            ), &Default::default()).unwrap(),
            PBR => draw.draw(&self.fsquad.0, &self.fsquad.1, &self.pbr, &uniform!(
                layera: self.layera.as_ref(),
                layerb: self.layerb.as_ref(),
                camera_pos: *camera.eye.as_ref(),
                albedo_tex: &self.pbrtex.albedo,
                roughness_tex: &self.pbrtex.roughness,
                metalness_tex: &self.pbrtex.metalness,
                normal_tex: &self.pbrtex.normal,
            ), &Default::default()).unwrap(),
            PHONG => draw.draw(&self.fsquad.0, &self.fsquad.1, &self.phong, &uniform!(
                layera: self.layera.as_ref(),
                layerb: self.layerb.as_ref(),
                pos_range: (0.0f32, 0.0f32),
                tex_range: (0.0f32, 1.0f32),
                norm_range: (0.0f32, 0.0f32),
            ), &Default::default()).unwrap(),
        }        
    }

    pub fn draw<S: Surface>(&mut self, draw: &mut S) {
        let (elapsed, delta) = self.update_draw_timer();

        if let Some((dx, dy)) = self.reset_mouse_drag(MouseButton::Left) {
            self.arcball.0 += dx * 1.2;
            self.arcball.1 += dy * 1.2;

            if self.arcball.1 < -HPI + 0.01 { self.arcball.1 = -HPI + 0.01; }
            if self.arcball.1 >  HPI - 0.01 { self.arcball.1 =  HPI - 0.01; }
        }

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

        let camera = self.get_camera();

        //=================//
        //                 //
        // TODO Draw stuff //
        //                 //
        //=================//

        draw.draw(&self.mesh.0, &self.mesh.1, &self.gbuff, &GbugffUniforms {
            view: camera.get_view(),
            proj: camera.get_proj(),
            ..Default::default()
        }, &params).unwrap();
    }   
}

fn duration_to_secs(dur: &Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 * 1e-9f64
}