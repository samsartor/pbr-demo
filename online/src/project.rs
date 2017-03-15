use glium::glutin::{Event, MouseButton, VirtualKeyCode, MouseScrollDelta, ElementState};
use glium::{Program, VertexBuffer, IndexBuffer, Surface, DrawParameters};
use glium::framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer};
use glium::texture::{DepthFormat, UncompressedFloatFormat, MipmapsOption, Texture2d};
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::GlutinFacade;
use glium;

use nalgebra::{Point3, Vector3};

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::rc::Rc;

use wavefront;
use shaders::{self, PhongUniforms, PhongLight};
use camera::{Camera, new_perspective};

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
    teapot_mesh: (VertexBuffer<wavefront::Vn>, IndexBuffer<u32>),
    phong: Program,
    // deferred data
    depth: Rc<DepthRenderBuffer>,
    layera: Rc<Texture2d>,
    layerb: Rc<Texture2d>,
}

impl<'a> Project<'a> {
    pub fn new(display: &'a GlutinFacade, start_size: (u32, u32)) -> Project<'a> {
        let teapot = wavefront::load_from_path("teapot.obj").unwrap();

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

        Project { 
            display: display,
            start: Instant::now(),
            last_draw: None,
            screen_size: start_size,
            mouse_pos: None,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            teapot_mesh: (teapot.vertex_buffer(display).unwrap(), teapot.index_buffer(display, PrimitiveType::TrianglesList).unwrap()),
            phong: shaders::wire_phong(display).unwrap(),
            depth: Rc::new(depth),
            layera: Rc::new(layera),
            layerb: Rc::new(layerb),
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
                // TODO
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
        match (self.mouse.get(&butt).cloned().unwrap_or(None), self.mouse_pos) {
            (Some((x0, y0)), Some((x1, y1))) => Some((x1 - x0, y1 - y0)),
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

    pub fn post<S: Surface>(&mut self, draw: &mut S) {

    }

    pub fn draw<S: Surface>(&mut self, draw: &mut S) {
        let (elapsed, delta) = self.update_draw_timer();

        draw.clear_color_and_depth((0.01, 0.01, 0.02, 1.0), 1.0);

        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let camera = new_perspective(
            Point3::new(3., 3., 3.), 
            Point3::new(0., 0.4, 0.),
            Vector3::new(0., 1., 0.),
            self.screen_size.0 as f32 / self.screen_size.1 as f32,
            25., 0.1, 1000.);

        let lights = [
            PhongLight {
                pos: Point3::new(4. * (elapsed * 1.6).sin() as f32, 2., 4. * (elapsed * 1.6).cos() as f32),
                color: [20., 5., 5.],
            },
            PhongLight {
                pos: Point3::new(0., 6., 0.),
                color: [10., 10., 15.],
            }
        ];


        //=================//
        //                 //
        // TODO Draw stuff //
        //                 //
        //=================//

        draw.draw(&self.teapot_mesh.0, &self.teapot_mesh.1, &self.phong, &PhongUniforms {
            view: camera.get_view(),
            proj: camera.get_proj(),
            ambient: [0.1; 3],
            line_width: 1.,
            material_spec: [0.4; 3],
            material_diff: [0.8, 0.8, 0.8],
            material_hard: 15.,
            lights: lights,
            show_wireframe: false,
            ..Default::default()
        }, &params).unwrap();
    }   
}

fn duration_to_secs(dur: &Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 * 1e-9f64
}