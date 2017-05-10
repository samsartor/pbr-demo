use cgmath::prelude::*;
use cgmath::{Point3, Matrix4, Deg, PerspectiveFov};
use image;
use gfx;
use gfx::traits::{FactoryExt};
use gfx::texture;
use gfx::format;
use gfx::handle::*;
use gfx_app::{self, ApplicationBase};
use winit::{self, Event};
use std::time::Instant;
use std::path::{Path, PathBuf};

use shaders;
use define::{self, VertexSlice};
use camera::{Camera, ArcBall};
use wavefront::{open_obj};

const AMBIENT: [f32; 4] = [0.03, 0.03, 0.03, 1.];

pub struct App<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    //=======//
    // Input //
    //=======//
    mouse_pos: Option<(i32, i32)>,
    show_floor: bool,
    orbit_diff: (f32, f32, f32),
    left_down: bool,
    cam: ArcBall<PerspectiveFov<f32>, Deg<f32>>,
    start_time: Instant,

    //===========//
    // App Stuff //
    //===========//
    encoder: gfx::Encoder<R, C>,

    //========//
    // Models //
    //========//
    mesh: VertexSlice<R, define::Vtnt>,
    quad: VertexSlice<R, define::V>,

    //===========//
    // Pipelines //
    //===========//
    mesh_deferred_pso: gfx::PipelineState<R, define::deferred::Meta>,
    pbr_pso: gfx::PipelineState<R, define::pbr::Meta>,

    //===============//
    // Pipeline Data //
    //===============//
    deferred_data: define::deferred::Data<R>,
    pbr_data: define::pbr::Data<R>,
}

struct ViewPair<R: gfx::Resources, T: gfx::format::Formatted> {
    resource: gfx::handle::ShaderResourceView<R, T::View>,
    target: gfx::handle::RenderTargetView<R, T>,
}

fn build_g_buf<R, C, F, T>(factory: &mut F, w: texture::Size, h: texture::Size) -> ViewPair<R, T>
    where F: gfx_app::Factory<R, CommandBuffer=C>,
          R: gfx::Resources + 'static,
          C: gfx::CommandBuffer<R> + Send + 'static,
          T: format::TextureFormat,
          T::Surface: format::RenderSurface,
          T::Channel: format::RenderChannel,
{
    let (_ , srv, rtv) = factory.create_render_target(w, h).unwrap();
    ViewPair {
        resource: srv,
        target: rtv,
    }
}

fn load_image<R, C, F, T, P>(factory: &mut F, path: P) -> (Texture<R, T::Surface>, ShaderResourceView<R, T::View>)
    where F: gfx_app::Factory<R, CommandBuffer=C>,
          R: gfx::Resources + 'static,
          C: gfx::CommandBuffer<R> + Send + 'static,
          P: AsRef<Path>,
          T: format::TextureFormat,
{
    use std::io::*;
    use std::fs::File;

    let image = image::load(BufReader::new(File::open(path).expect("Image not found")), image::PNG).unwrap().to_rgba();
    let dim = image.dimensions();

    factory.create_texture_immutable_u8::<T>(
        texture::Kind::D2(dim.0 as u16, dim.1 as u16, texture::AaMode::Single),
        &[&image]
    ).expect("Could not upload texture")  // TODO: Result
}

fn get_args() -> PathBuf {
    use clap::{App, Arg};

    let args = App::new("PBR Demo")
        .author("Sam Sartor <ssartor@mines.edu>")
        .author("Daichi Jameson <djameson@mines.edu>")
        .arg(Arg::with_name("object")
            .short("o")
            .long("object")
            .help("directory containing model.obj and PBR textures")
            .required(true)
            .takes_value(true))
    .get_matches();

    (PathBuf::from(args.value_of("object").unwrap()))
}

impl<R, C> ApplicationBase<R, C> for App<R, C> where
    R: gfx::Resources + 'static,
    C: gfx::CommandBuffer<R> + Send + 'static,
{
    fn new<F>(factory: &mut F, _: gfx_app::shade::Backend, window_targets: gfx_app::WindowTargets<R>) -> Self
    where F: gfx_app::Factory<R, CommandBuffer=C>,
    {
        let directory = get_args();

        let dim = window_targets.color.get_dimensions();

        let mesh = open_obj(directory.join("model.obj"), factory).unwrap();

        use self::format::*;

        let normal_tex = load_image::<_, _, _, (R8_G8_B8_A8, Unorm), _>(factory, directory.join("normal.png"));
        let albedo_tex = load_image::<_, _, _, (R8_G8_B8_A8, Unorm), _>(factory, directory.join("albedo.png"));
        let metal_tex = load_image::<_, _, _, (R8_G8_B8_A8, Unorm), _>(factory, directory.join("metalness.png"));
        let rough_tex = load_image::<_, _, _, (R8_G8_B8_A8, Unorm), _>(factory, directory.join("roughness.png"));

        let layer_a = build_g_buf(factory, dim.0, dim.1);
        let layer_b = build_g_buf(factory, dim.0, dim.1);

        let (_, _, depth) = factory.create_depth_stencil(dim.0, dim.1).unwrap();

        let mesh_deferred_pso = {
            let shaders = shaders::deferred(factory).unwrap();
            factory.create_pipeline_state(
                &shaders,
                gfx::Primitive::TriangleList,
                gfx::state::Rasterizer::new_fill(),
                define::deferred::new()
            ).unwrap()
        };

        let pbr_pso = {
            let shaders = shaders::pbr(factory).unwrap();
            factory.create_pipeline_state(
                &shaders,
                gfx::Primitive::TriangleList,
                gfx::state::Rasterizer::new_fill(),
                define::pbr::new()
            ).unwrap()
        };

        let gbuf_sampler = factory.create_sampler(texture::SamplerInfo::new(
            texture::FilterMethod::Scale,
            texture::WrapMode::Clamp,
        ));

        let sampler = factory.create_sampler(texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Tile,
        ));

        let deferred_data = define::deferred::Data {
            verts: mesh.0.clone(),
            transform: factory.create_constant_buffer(1),
            layer_a: layer_a.target.clone(),
            layer_b: layer_b.target.clone(),
            normal_tex: (normal_tex.1.clone(), sampler.clone()),
            depth: depth.clone()
        };

        let quad = {
            use define::V;

            factory.create_vertex_buffer_with_slice(
                &[V { a_pos: [-1., -1., 0.] },
                  V { a_pos: [-1.,  1., 0.] },
                  V { a_pos: [ 1., -1., 0.] },
                  V { a_pos: [ 1.,  1., 0.] }],
                &[0u16, 1, 2, 3, 1, 2][..],
            )
        };

        let pbr_data = define::pbr::Data {
            verts: quad.0.clone(),
            live: factory.create_constant_buffer(1),
            layer_a: (layer_a.resource.clone(), gbuf_sampler.clone()),
            layer_b: (layer_b.resource.clone(), gbuf_sampler.clone()),
            albedo: (albedo_tex.1.clone(), sampler.clone()),
            metalness: (metal_tex.1.clone(), sampler.clone()),
            roughness: (rough_tex.1.clone(), sampler.clone()),
            color: window_targets.color.clone(),  
        };

        App {
            mouse_pos: None,
            show_floor: true,
            orbit_diff: (0., 0., 0.),
            left_down: false,
            cam: ArcBall {
                origin: Point3::new(0., 0., 0.),
                theta: Deg(45.),
                phi: Deg(35.264),
                dist: 4.,
                projection: PerspectiveFov {
                    fovy: Deg(35.).into(),
                    aspect: window_targets.aspect_ratio, 
                    near: 0.1, far: 100.
                },
            },
            start_time: Instant::now(),

            encoder: factory.create_encoder(),

            mesh: mesh,
            quad: quad,

            mesh_deferred_pso: mesh_deferred_pso,
            pbr_pso: pbr_pso,

            deferred_data: deferred_data,
            pbr_data: pbr_data,
        }
    }

    fn render<D>(&mut self, device: &mut D) where
        D: gfx::Device<Resources=R, CommandBuffer=C>
    {
        // camera stuff
        self.cam.theta += Deg(self.orbit_diff.0 * 0.2);
        self.cam.phi += Deg(self.orbit_diff.1 * 0.2);
        if self.cam.phi < Deg(-89.) { self.cam.phi = Deg(-89.) }
        if self.cam.phi > Deg(89.) { self.cam.phi = Deg(89.) }

        self.cam.dist *= (self.orbit_diff.2 * -0.1).exp();
        self.orbit_diff = (0., 0., 0.);

        let camera = self.cam.to_camera();

        let elapsed = self.start_time.elapsed();
        let elapsed = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9f64;

        // clear screen
        self.encoder.clear(&self.pbr_data.color, AMBIENT);
        self.encoder.clear(&self.deferred_data.layer_a, [0.; 4]);
        self.encoder.clear(&self.deferred_data.layer_b, [0.; 4]);
        self.encoder.clear_depth(&self.deferred_data.depth, self.cam.projection.far);

        self.encoder.update_constant_buffer(&self.deferred_data.transform, &define::TransformBlock {
            model: Matrix4::identity().into(),
            view: camera.get_view().into(),
            proj: camera.get_proj().into(),
        });

        self.encoder.draw(&self.mesh.1, &self.mesh_deferred_pso, &self.deferred_data);

        self.encoder.update_constant_buffer(&self.pbr_data.live, &define::LiveBlock {
            eye_pos: camera.get_eye().to_vec().extend(1.).into(),
            exposure: 0.06,
            gamma: 2.2,
            time: elapsed as f32,
        });

        self.encoder.draw(&self.quad.1, &self.pbr_pso, &self.pbr_data);

        // send to GPU
        self.encoder.flush(device);
    }

    fn get_exit_key() -> Option<winit::VirtualKeyCode> {
        Some(winit::VirtualKeyCode::Escape)
    }

    fn on(&mut self, event: Event) {
        use self::Event::*;
        use winit::ElementState::*;

        match event {
            KeyboardInput(state, _, Some(code)) => {
                use winit::VirtualKeyCode::*;

                match (state, code) {
                    (Pressed, F) => self.show_floor = !self.show_floor,
                    // (Pressed, M) => self.model_index = (self.model_index + 1) % self.models.len(),
                    _ => ()
                }
            },
            MouseMoved(x, y) => {
                let mut dx = 0;
                let mut dy = 0;

                    let p = (x, y);

                if let Some((x0, y0)) = self.mouse_pos {
                    dx = p.0 - x0;
                    dy = p.1 - y0;
                }

                self.mouse_pos = Some(p);

                if self.left_down {
                    self.orbit_diff.0 += dx as f32;
                    self.orbit_diff.1 += dy as f32;
                }
            },
            MouseLeft => self.mouse_pos = None,
            MouseInput(state, butt) => {
                use winit::MouseButton::*;

                match (state, butt) {
                    (Pressed, Left) => {
                        self.left_down = true;
                    },
                    (Released, Left) => {
                        self.left_down = false;
                    },
                    _ => (),

                }
            }
            MouseWheel(winit::MouseScrollDelta::LineDelta(_, y), _) => {
                self.orbit_diff.2 += y;
            },
            _ => (),
        }
    }

    fn on_resize<F>(&mut self, factory: &mut F, window_targets: gfx_app::WindowTargets<R>)
    where F: gfx_app::Factory<R, CommandBuffer=C>
    {
        let (w, h, _, _) = window_targets.color.get_dimensions();

        let layer_a = build_g_buf(factory, w, h);
        let layer_b = build_g_buf(factory, w, h);

        self.deferred_data.layer_a = layer_a.target.clone();
        self.deferred_data.layer_b = layer_b.target.clone();

        let (_, _, depth) = factory.create_depth_stencil(w, h).unwrap();
        self.deferred_data.depth = depth;

        self.pbr_data.layer_a.0 = layer_a.resource.clone();
        self.pbr_data.layer_b.0 = layer_b.resource.clone();

        self.pbr_data.color = window_targets.color.clone();

        self.cam.projection.aspect = window_targets.aspect_ratio;
    }
}