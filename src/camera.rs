use cgmath::{Quaternion, Rotation, Rotation3, EuclideanSpace, Array, InnerSpace};
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
    turn_rate: f32,

    moving_left: bool,
    moving_right: bool,
    moving_forward: bool,
    moving_backward: bool,
    moving_up: bool,
    moving_down: bool,
    turning_left: bool,
    turning_right: bool,
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
            moving_left: false, 
            moving_right: false,
            moving_forward: false,
            moving_backward: false,
            moving_up: false,
            moving_down: false,
            turning_left: false,
            turning_right: false
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
                    VirtualKeyCode::Q =>
                    {
                        self.turning_left = is_pressed;
                        true
                    },
                    VirtualKeyCode::E =>
                    {
                        self.turning_right = is_pressed;
                        true
                    },
                    VirtualKeyCode::W =>
                    {
                        self.moving_forward = is_pressed;
                        true
                    },
                    VirtualKeyCode::S =>
                    {
                        self.moving_backward = is_pressed;
                        true
                    },
                    VirtualKeyCode::A => 
                    {
                        self.moving_left = is_pressed;
                        true
                    },
                    VirtualKeyCode::D => 
                    {
                        self.moving_right = is_pressed;
                        true
                    },
                    VirtualKeyCode::LShift => 
                    {
                        self.moving_down = is_pressed;
                        true
                    },
                    VirtualKeyCode::Space => 
                    {
                        self.moving_up = is_pressed;
                        true
                    }
                    _ => false
                }
            },

            _ => false
        }
    }

    pub fn update(&mut self, delta_time: f32)
    {
        self.rotate_camera(delta_time);
        self.move_camera(delta_time);
    }

    fn move_camera(&mut self, delta_time: f32)
    {
        let mut move_dir = Vec3::from_value(0.0);
        if self.moving_forward { move_dir.z += -1.0; }
        if self.moving_backward { move_dir.z += 1.0; }
        if self.moving_left { move_dir.x += -1.0; }
        if self.moving_right { move_dir.x += 1.0; }
        if self.moving_up { move_dir.y += 1.0; }
        if self.moving_down { move_dir.y += -1.0; }

        if move_dir.x != 0.0 || move_dir.y != 0.0 || move_dir.z != 0.0
        {
            move_dir = move_dir.normalize() * delta_time * self.speed;
        }

        self.camera.eye += move_dir;
        self.camera.target += move_dir;
    }

    fn rotate_camera(&mut self, delta_time: f32)
    {
        let mut turn_dir = 0.;
        if self.turning_left { turn_dir += -1.; }
        if self.turning_right { turn_dir += 1.; }

        let rotation = Quaternion::from_angle_y(cgmath::Deg(turn_dir * self.turn_rate * delta_time));

        let mut target_relative = self.camera.target - self.camera.eye;
        target_relative = rotation.rotate_vector(target_relative);
        let target_vec = target_relative + self.camera.eye.to_vec();
        self.camera.target = Point3D::new(target_vec.x, target_vec.y, target_vec.z);
    }
}