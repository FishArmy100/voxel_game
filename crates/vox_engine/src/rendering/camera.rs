use cgmath::{Quaternion, Rotation, Rotation3, EuclideanSpace, Array, InnerSpace, Deg, Transform, SquareMatrix};

use crate::{math::*, app::input::{FrameState, KeyCode}};

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
    pub fn new() -> Self 
    {
        Self 
        {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) 
    {
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
    current_vertical_look: f32,
    max_vertical_look: f32
}

impl CameraEntity
{
    pub fn new(camera: Camera, speed: f32, turn_rate: f32, max_vertical_look: f32) -> CameraEntity
    {
        CameraEntity 
        {
            camera, 
            speed, 
            turn_rate,
            current_vertical_look: 0.0,
            max_vertical_look
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
        let forward = -(Vec3::new(self.camera.eye.x, 0.0, self.camera.eye.z) - Vec3::new(self.camera.target.x, 0.0, self.camera.target.z)).normalize();
        let left = Quaternion::from_angle_y(Deg(90.0)).rotate_vector(forward).normalize();

        let mut move_dir = Vec3::from_value(0.0);

        if frame_state.is_key_down(KeyCode::W) { move_dir += forward; }
        if frame_state.is_key_down(KeyCode::S) { move_dir += -forward; }
        if frame_state.is_key_down(KeyCode::A) { move_dir += left; }
        if frame_state.is_key_down(KeyCode::D) { move_dir += -left; }

        if frame_state.is_key_down(KeyCode::Space) { move_dir.y += 1.0; }
        if frame_state.is_key_down(KeyCode::LShift) { move_dir.y += -1.0; }

        if move_dir.x != 0.0 || move_dir.y != 0.0 || move_dir.z != 0.0
        {
            move_dir = move_dir.normalize() * frame_state.delta_time() * self.speed;
        }

        self.camera.eye += move_dir;
        self.camera.target += move_dir;
    }

    fn rotate_camera(&mut self, frame_state: &FrameState)
    {
        self.current_vertical_look = (self.current_vertical_look + frame_state.mouse_delta().y * self.turn_rate * frame_state.delta_time()).clamp(-self.max_vertical_look, self.max_vertical_look);

        let horizontal_rotation = Quaternion::from_angle_y(Deg(-frame_state.mouse_delta().x * self.turn_rate * frame_state.delta_time()));

        let forward = -(Vec3::new(self.camera.eye.x, 0.0, self.camera.eye.z) - Vec3::new(self.camera.target.x, 0.0, self.camera.target.z)).normalize();
        let right = Quaternion::from_angle_y(Deg(90.0)).rotate_vector(forward).normalize();

        let vertical_rotation = Quaternion::from_axis_angle(right, Deg(self.current_vertical_look));
        let rotation = vertical_rotation * horizontal_rotation;

        let target_relative = rotation.rotate_vector(forward);

        let target_vec = target_relative + self.camera.eye.to_vec();
        self.camera.target = Point3D::new(target_vec.x, target_vec.y, target_vec.z);
    }
}