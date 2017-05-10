#![allow(dead_code)]

use cgmath::prelude::*;
use cgmath::{Perspective, PerspectiveFov, Ortho, Rad, Matrix4, Point3, Vector3, Angle, vec3};

pub type BasicPerspCamera = BasicCamera<Perspective<f32>>;
pub type BasicOrthoCamera = BasicCamera<Ortho<f32>>;
pub type BasicCustomCamera = BasicCamera<CustomProjection>;
pub type DirectPerspCamera = DirectCamera<Perspective<f32>>;
pub type DirectOrthoCamera = DirectCamera<Ortho<f32>>;
pub type DirectCustomCamera = DirectCamera<CustomProjection>;

pub trait Camera {
    fn get_eye(&self) -> Point3<f32>;
    fn get_view(&self) -> Matrix4<f32>;
    fn get_proj(&self) -> Matrix4<f32>;
    fn get_clip(&self) -> (f32, f32);
}

pub trait Projection: Clone + Copy {
    fn matrix(self) -> Matrix4<f32>;
    fn clip(&self) -> (f32, f32);
}

#[derive(Debug, Clone, Copy)]
pub struct CustomProjection {
    near: f32,
    far: f32,
    mat: Matrix4<f32>,
}

impl CustomProjection {
    pub fn new(mat: Matrix4<f32>, near: f32, far: f32) -> CustomProjection {
        CustomProjection {
            mat: mat,
            near: near,
            far: far,
        }
    }
}

impl Projection for CustomProjection {
    fn matrix(self) -> Matrix4<f32> {
        self.mat
    }
    fn clip(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

impl Projection for Perspective<f32> {
    fn matrix(self) -> Matrix4<f32> {
        self.into()
    }

    fn clip(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

impl Projection for PerspectiveFov<f32> {
    fn matrix(self) -> Matrix4<f32> {
        self.to_perspective().into()
    }

    fn clip(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

impl Projection for Ortho<f32> {
    fn matrix(self) -> Matrix4<f32> {
        self.into()
    }
    fn clip(&self) -> (f32, f32) {
        (self.near, self.far)
    }
}

pub struct BasicCamera<P: Projection> {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub projection: P,
}

pub fn new_perspective(eye: Point3<f32>,
                       target: Point3<f32>,
                       up: Vector3<f32>,
                       aspect: f32,
                       fovy: Rad<f32>,
                       near: f32,
                       far: f32)
                       -> BasicPerspCamera {
    BasicCamera {
        eye: eye,
        target: target,
        up: up,
        projection: PerspectiveFov { fovy, aspect, near, far }.to_perspective(),
    }
}

pub fn new_orthographic(eye: Point3<f32>,
                        target: Point3<f32>,
                        up: Vector3<f32>,
                        left: f32,
                        right: f32,
                        top: f32,
                        bottom: f32,
                        near: f32,
                        far: f32)
                        -> BasicOrthoCamera {
    BasicCamera {
        eye: eye,
        target: target,
        up: up,
        projection: Ortho { left, right, top, bottom, near, far },
    }
}

impl<P: Projection> BasicCamera<P> {
    pub fn to_direct(&self) -> DirectCamera<P> {
        new_direct(self.get_view(), self.projection)
    }
}

impl<P: Projection> Camera for BasicCamera<P> {
    fn get_eye(&self) -> Point3<f32> {
        self.eye
    }

    fn get_view(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.eye, self.target, self.up)
    }

    fn get_proj(&self) -> Matrix4<f32> {
        self.projection.matrix()
    }

    fn get_clip(&self) -> (f32, f32) {
        self.projection.clip()
    }
}

pub struct DirectCamera<P: Projection> {
    pub eye: Matrix4<f32>,
    pub projection: P,
}

pub fn new_direct<P: Projection>(eye: Matrix4<f32>, proj: P) -> DirectCamera<P> {
    DirectCamera {
        eye: eye,
        projection: proj,
    }
}

impl<P: Projection> Camera for DirectCamera<P> {
    fn get_eye(&self) -> Point3<f32> {
        self.eye.invert().expect("View matrix is not ivertable").transform_point(Point3::new(0., 0., 0.))
    }

    fn get_view(&self) -> Matrix4<f32> {
        self.eye
    }

    fn get_proj(&self) -> Matrix4<f32> {
        self.projection.matrix()
    }

    fn get_clip(&self) -> (f32, f32) {
        self.projection.clip()
    }
}

pub fn new_direct_custom(eye: Matrix4<f32>,
                         proj: Matrix4<f32>,
                         near: f32,
                         far: f32)
                         -> DirectCustomCamera {
    DirectCamera {
        eye: eye,
        projection: CustomProjection {
            near: near,
            far: far,
            mat: proj,
        },
    }
}

pub struct ArcBall<P: Projection, A: Angle> {
    pub origin: Point3<f32>,
    pub theta: A,
    pub phi: A,
    pub dist: f32,
    pub projection: P
}

impl<P: Projection, A: Angle<Unitless=f32>> ArcBall<P, A>
{
    pub fn to_camera(&self) -> BasicCamera<P> {
        let phicos = self.phi.cos();

        BasicCamera {
            eye: self.origin + vec3(self.theta.cos() * phicos, self.phi.sin(), self.theta.sin() * phicos) * self.dist,
            target: self.origin,
            up: vec3(0., 1., 0.),
            projection: self.projection,
        }
    }
}