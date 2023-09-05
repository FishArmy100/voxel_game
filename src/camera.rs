use cgmath::{Quaternion, Rotation, Rotation3, EuclideanSpace, Array, InnerSpace};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::{math::*, application::FrameState};

#[derive(Debug, Clone)]
pub struct Camera 
{
    pub eye: Point3D<f32>,
    pub target: Point3D<f32>,
    pub up: Vec3<f32>,
    pub aspect: f32,
    pub fov: f32, 
    pub near: f32,
    pub far: f32
}

impl Camera 
{
    pub fn build_view_projection_matrix(&self) -> Mat4x4<f32>
    {
        let view = Mat4x4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform 
{
    view_proj: Mat4x4<f32>,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

#[derive(Debug, Clone)]
pub struct CameraEntity
{
    camera: Camera,
    speed: f32,
    turn_rate: f32,
}

impl CameraEntity
{
    pub fn new(camera: Camera, speed: f32, turn_rate: f32) -> CameraEntity
    {
        CameraEntity 
        {
            camera, 
            speed, 
            turn_rate,
        }
    }

    pub fn camera(&self) -> &Camera {&self.camera}
    pub fn mut_camera(&mut self) -> &mut Camera {&mut self.camera}
    pub fn update(&mut self, frame_state: &FrameState)
    {
        self.rotate_camera(frame_state);
        self.move_camera(frame_state);
    }

    fn move_camera(&mut self, frame_state: &FrameState)
    {
        let mut move_dir = Vec3::from_value(0.0);
        if frame_state.is_key_down(VirtualKeyCode::W) { move_dir.z += -1.0; }
        if frame_state.is_key_down(VirtualKeyCode::S) { move_dir.z += 1.0; }
        if frame_state.is_key_down(VirtualKeyCode::A) { move_dir.x += -1.0; }
        if frame_state.is_key_down(VirtualKeyCode::D) { move_dir.x += 1.0; }
        if frame_state.is_key_down(VirtualKeyCode::Space) { move_dir.y += 1.0; }
        if frame_state.is_key_down(VirtualKeyCode::LShift) { move_dir.y += -1.0; }

        if move_dir.x != 0.0 || move_dir.y != 0.0 || move_dir.z != 0.0
        {
            move_dir = move_dir.normalize() * frame_state.delta_time() * self.speed;
        }

        self.camera.eye += move_dir;
        self.camera.target += move_dir;
    }

    fn rotate_camera(&mut self, frame_state: &FrameState)
    {
        let mut turn_dir = 0.;
        if frame_state.is_key_down(VirtualKeyCode::Q) { turn_dir += -1.; }
        if frame_state.is_key_down(VirtualKeyCode::E) { turn_dir += 1.; }

        let rotation = Quaternion::from_angle_y(cgmath::Deg(turn_dir * self.turn_rate * frame_state.delta_time()));

        let mut target_relative = self.camera.target - self.camera.eye;
        target_relative = rotation.rotate_vector(target_relative);
        let target_vec = target_relative + self.camera.eye.to_vec();
        self.camera.target = Point3D::new(target_vec.x, target_vec.y, target_vec.z);
    }
}