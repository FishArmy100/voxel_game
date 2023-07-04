use cgmath::{Quaternion, Rotation, Rotation3};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::math::*;

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

pub struct CameraEntity
{
    camera: Camera,
    speed: f32,
    moving_left: bool,
    moving_right: bool
}

impl CameraEntity
{
    pub fn new(camera: Camera, speed: f32) -> CameraEntity
    {
        CameraEntity 
        {
            camera, 
            speed, 
            moving_left: false, 
            moving_right: false 
        }
    }

    pub fn camera(&self) -> &Camera {&self.camera}
    pub fn mut_camera(&mut self) -> &mut Camera {&mut self.camera}

    pub fn on_event(&mut self, event: &WindowEvent) -> bool
    {
        match event
        {
            WindowEvent::KeyboardInput 
            { 
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => 
            {
                let is_pressed = *state == ElementState::Pressed;
                match keycode 
                {
                    VirtualKeyCode::A | VirtualKeyCode::Left =>
                    {
                        self.moving_left = is_pressed;
                        true
                    },
                    VirtualKeyCode::D | VirtualKeyCode::Right =>
                    {
                        self.moving_right = is_pressed;
                        true
                    },
                    _ => false
                }
            },

            _ => false
        }
    }

    pub fn update(&mut self, delta_time: f32)
    {
        let pos = self.camera.eye;
        let dir = 
            if self.moving_left 
            {
                1.
            } 
            else 
            {
                0.
            } 
                + 
            if self.moving_right 
            {
                -1.
            } 
            else 
            {
                0.
            };

        let rotation = Quaternion::from_angle_y(cgmath::Deg(dir * self.speed * delta_time));

        self.camera.eye = rotation.rotate_point(pos);
    }
}