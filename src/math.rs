
pub type Vec2<T> = cgmath::Vector2<T>;
pub type Vec3<T> = cgmath::Vector3<T>;
pub type Vec4<T> = cgmath::Vector4<T>;

pub type Point3D<T> = cgmath::Point3<T>;
pub type Point2D<T> = cgmath::Point2<T>;

pub type Mat4x4<T> = cgmath::Matrix4<T>;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);
