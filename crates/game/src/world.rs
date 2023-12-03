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
            up: Vec3::Y,
            aspect: 1.0, // is set on update
            fov: 45.0,
            near: 0.1,
            far: 100000.0
        };

        let camera_entity = CameraEntity::new(camera, 20.0, 50.0, 80.0);

        Self 
        {  
            main_camera: camera_entity
        }
    }

    pub fn on_update(&mut self, frame_state: &FrameState)
    {
        let aspect = frame_state.window_size().x as f32 / frame_state.window_size().y as f32;
        self.main_camera.mut_camera().aspect = aspect;
        self.main_camera.update(frame_state);
    }

    pub fn on_gui(&mut self, frame_state: &FrameState, gui_context: &egui::Context)
    {
        GuiWindow::new("Basic Window")
            .resizable(true)
            .show(gui_context, |ui| {
                ui.label("Hello World!");
                ui.label(format!("Frame time: {}ms", frame_state.delta_time() * 1000.0));
                ui.label(RichText::new("With new stuff"));
            });
    }
}