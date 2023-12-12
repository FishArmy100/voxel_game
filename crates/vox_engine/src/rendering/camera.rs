use crate::{math::*, app::input::{FrameState, KeyCode}};
use glam::Quat;
pub use vox_core::camera::Camera;

#[derive(Clone)]
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
        let left = Quat::from_axis_angle(Vec3::Y, (90.0 as f32).to_radians()) * forward;

        let mut move_dir = Vec3::ZERO;

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

        self.camera.eye += Vec3A::from(move_dir);
        self.camera.target += Vec3A::from(move_dir);
    }

    fn rotate_camera(&mut self, frame_state: &FrameState)
    {
        self.current_vertical_look = (self.current_vertical_look + frame_state.mouse_delta().y * self.turn_rate * frame_state.delta_time()).clamp(-self.max_vertical_look, self.max_vertical_look);
        let horizontal_angle = -frame_state.mouse_delta().x * self.turn_rate * frame_state.delta_time();
        
        let horizontal_rotation = Quat::from_axis_angle(Vec3::Y, horizontal_angle.to_radians());

        let forward = -(Vec3::new(self.camera.eye.x, 0.0, self.camera.eye.z) - Vec3::new(self.camera.target.x, 0.0, self.camera.target.z)).normalize();
        let right = -forward.cross(Vec3::Y).normalize();

        let vertical_rotation = Quat::from_axis_angle(right, self.current_vertical_look.to_radians());
        
        let rotation = vertical_rotation * horizontal_rotation;

        let target_relative = rotation * forward;

        let target_vec = Vec3A::from(target_relative) + self.camera.eye;
        self.camera.target = Vec3A::new(target_vec.x, target_vec.y, target_vec.z);
    }
}