use vox_engine::app::input::FrameState;
use vox_engine::egui::{self, Window, RichText};
use vox_engine::math::Vec3;
use vox_engine::rendering::camera::{Camera, CameraEntity};
use vox_engine::rendering::gui::GuiWindow;

pub struct GameWorld
{
    pub main_camera: CameraEntity
}

impl GameWorld
{
    pub fn new() -> Self 
    {
        let camera = Camera
        {
            eye: (0.0, 0.0, 0.0).into(),
            target: (0.0, 0.0, 1.0).into(),
            fov: 45.0,
        };

        let camera_entity = CameraEntity::new(camera, 20.0, 50.0, 80.0);

        Self 
        {  
            main_camera: camera_entity
        }
    }

    pub fn on_update(&mut self, frame_state: &FrameState)
    {
        self.main_camera.update(frame_state);
    }

    pub fn on_gui(&mut self, frame_state: &FrameState, gui_context: &egui::Context)
    {
        let camera_pos = self.main_camera.camera().eye;

        GuiWindow::new("Basic Window")
            .resizable(true)
            .show(gui_context, |ui| {
                ui.label("Hello World!");
                ui.label(format!("Frame time: {:.2}ms", frame_state.delta_time() * 1000.0));
                ui.label(format!("Camera: [{:.2}, {:.2}, {:.2}]", camera_pos.x, camera_pos.y, camera_pos.z));
            });
    }
}