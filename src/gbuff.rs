use glium::framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer};
use glium::texture::{DepthFormat, UncompressedFloatFormat, MipmapsOption, Texture2d};
use glium::backend::glutin_backend::GlutinFacade;

use std::rc::Rc;
use std::mem::transmute;

pub struct Gbuff {
    display: Rc<GlutinFacade>,
    depth: Box<DepthRenderBuffer>,
    layera: Box<Texture2d>,
    layerb: Box<Texture2d>,
    fbo: *mut MultiOutputFrameBuffer<'static>,
}

impl Gbuff {
    pub fn new(display: Rc<GlutinFacade>, size: (u32, u32)) -> Gbuff {
        let layera = Box::new(Texture2d::empty_with_format(
            display.as_ref(), 
            UncompressedFloatFormat::F32F32F32F32,
            MipmapsOption::NoMipmap,
            size.0, size.1).unwrap());

        let layerb = Box::new(Texture2d::empty_with_format(
            display.as_ref(), 
            UncompressedFloatFormat::F32F32F32F32,
            MipmapsOption::NoMipmap,
            size.0, size.1).unwrap());

        let depth = Box::new(DepthRenderBuffer::new(
            display.as_ref(),
            DepthFormat::I24,
            size.0, size.1).unwrap());

        let fbo: *mut MultiOutputFrameBuffer<'static> = unsafe {
            transmute(Box::into_raw(Box::new( // allocate and transmute
                MultiOutputFrameBuffer::with_depth_buffer(
                    display.as_ref(), 
                    vec![("layera", &*layera), ("layerb", &*layerb)], 
                    &*depth).unwrap()))) };

        Gbuff {
            display: display,
            depth: depth,
            layera: layera,
            layerb: layerb,
            fbo: fbo,
        }
    }

    pub fn get_mut_fbo<'a>(&'a mut self) -> &'a mut MultiOutputFrameBuffer<'a> {
        unsafe {
            let tm: *mut MultiOutputFrameBuffer<'a> = transmute(self.fbo);
            &mut *tm
        }
    }

    pub fn get_fbo<'a>(&'a self) -> &'a MultiOutputFrameBuffer<'a> {
        unsafe {
            let tm: *mut MultiOutputFrameBuffer<'a> = transmute(self.fbo);
            &mut *tm
        }
    }

    pub fn get_depth(&self) -> &DepthRenderBuffer {
        &*self.depth
    }

    pub fn get_layera(&self) -> &Texture2d {
        &*self.layera
    }

    pub fn get_layerb(&self) -> &Texture2d {
        &*self.layerb
    }
}

impl Drop for Gbuff {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(self.fbo); // easy way to drop
        }
    }
}