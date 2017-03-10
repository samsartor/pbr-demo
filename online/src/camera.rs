use nalgebra::{Isometry3, Point3, Vector3, Matrix4, PerspectiveMatrix3, OrthographicMatrix3, ToHomogeneous};

pub type BasicPerspCamera = BasicCamera<PerspectiveMatrix3<f32>>;
pub type BasicOrthoCamera = BasicCamera<OrthographicMatrix3<f32>>;
pub type BasicCustomCamera = BasicCamera<CustomProjection>;
pub type DirectPerspCamera = DirectCamera<PerspectiveMatrix3<f32>>;
pub type DirectOrthoCamera = DirectCamera<OrthographicMatrix3<f32>>;
pub type DirectCustomCamera = DirectCamera<CustomProjection>;

pub trait Camera {
    fn get_view(&self) -> Matrix4<f32>;
    fn get_proj(&self) -> Matrix4<f32>;
    fn get_clip(&self) -> (f32, f32);
}

pub trait Projection: Clone + Copy {
    fn matrix(&self) -> Matrix4<f32>;
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
    fn matrix(&self) -> Matrix4<f32> { self.mat }
    fn clip(&self) -> (f32, f32) { (self.near, self.far) }
}

impl Projection for PerspectiveMatrix3<f32> {
    fn matrix(&self) -> Matrix4<f32> { self.to_matrix() }
    fn clip(&self) -> (f32, f32) { (self.znear(), self.zfar()) }
}

impl Projection for OrthographicMatrix3<f32> {
    fn matrix(&self) -> Matrix4<f32> { self.to_matrix() }
    fn clip(&self) -> (f32, f32) { (self.znear(), self.zfar()) }
}

pub struct BasicCamera<P: Projection> {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub projection: P,
}

pub fn new_perspective(eye: Point3<f32>, target: Point3<f32>, up: Vector3<f32>,
                       aspect: f32, fovy: f32, near: f32, far: f32) 
-> BasicPerspCamera {
    BasicCamera {
        eye: eye,
        target: target,
        up: up,
        projection: PerspectiveMatrix3::new(
            aspect, fovy, near, far
        ),
    }
}

pub fn new_orthographic(eye: Point3<f32>, target: Point3<f32>, up: Vector3<f32>,
                        left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32)
-> BasicOrthoCamera {
    BasicCamera {
        eye: eye,
        target: target,
        up: up,
        projection: OrthographicMatrix3::new(
            left, right, top, bottom, near, far
        ),
    }
}

impl<P: Projection> BasicCamera<P> {
    pub fn to_direct(&self) -> DirectCamera<P> {
        DirectCamera::new(self.get_view(), self.projection)
    }
}

impl<P: Projection> Camera for BasicCamera<P> {
    fn get_view(&self) -> Matrix4<f32> {
        Isometry3::look_at_rh(&self.eye, &self.target, &self.up).to_homogeneous()
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

impl<P: Projection> DirectCamera<P> {
    pub fn new(eye: Matrix4<f32>, proj: P) -> DirectCamera<P> {
        DirectCamera {
            eye: eye,
            projection: proj,
        }
    }
}

impl<P: Projection> Camera for DirectCamera<P> {
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

impl DirectCustomCamera {
    pub fn new_direct_custom(eye: Matrix4<f32>, proj: Matrix4<f32>, near: f32, far: f32) -> DirectCustomCamera {
        DirectCamera {
            eye: eye,
            projection: CustomProjection {
                near: near,
                far: far,
                mat: proj,
            },
        }
    }
}