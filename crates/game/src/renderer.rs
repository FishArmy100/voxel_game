
use std::sync::Arc;

use vox_engine::app::WinitWindow;
use vox_engine::app::input::FrameState;
use vox_engine::gpu_utils::WgpuState;
use vox_engine::math::Color;
use vox_engine::rendering::Renderer;
use vox_engine::rendering::camera::Camera;
use vox_engine::voxel::VoxelRenderer;
use vox_engine::wgpu::SurfaceError;
use vox_engine::winit::event_loop::EventLoop;
use vox_engine::winit::event::Event;
use vox_engine::rendering::gui::{GuiRenderer, GuiRendererDescriptor};
use vox_engine::glam::UVec2;

use crate::GUI_SAVE_PATH;
use crate::world::GameWorld;

#[derive(Clone, Copy)]
pub struct RenderData <'a>
{
    pub frame_state: &'a FrameState,
    pub gpu_state: &'a WgpuState,
}

pub struct GameRenderer
{
    renderer: Renderer,
    gui_renderer: GuiRenderer,
    voxel_renderer: VoxelRenderer,
}

impl GameRenderer
{
    pub fn new<T>(gpu_state: &WgpuState, event_loop: &EventLoop<T>, window: Arc<WinitWindow>, camera: &Camera) -> Self
        where T : Sized
    {
        let renderer = Renderer::new(gpu_state.device().clone(), gpu_state.surface().clone(), gpu_state.queue().clone(), &gpu_state.surface_config(), Color::new(0.2, 0.2, 0.2, 1.0));
        
        let mut gui_renderer = GuiRenderer::new(GuiRendererDescriptor { 
            event_loop, 
            device: &gpu_state.device(), 
            rt_format: gpu_state.surface_config().format,
            window
        });

        gui_renderer.load(GUI_SAVE_PATH);

        let voxel_renderer = VoxelRenderer::new(&gpu_state, camera);

        Self 
        { 
            renderer, 
            gui_renderer, 
            voxel_renderer,
        }
    }

    pub fn on_exit(&self)
    {
        self.gui_renderer.save(GUI_SAVE_PATH);
    }

    pub fn on_resize(&mut self, new_size: UVec2, gpu_state: &WgpuState)
    {
        if new_size.x > 0 && new_size.y > 0
        {
            self.renderer.resize(&gpu_state.surface_config());
            self.voxel_renderer.resize(gpu_state.queue(), gpu_state.device(), gpu_state.surface_config());
        }
    }

    pub fn gui_handle_event<'a, T>(&mut self, event: &Event<'a, T>) -> bool 
    {
        self.gui_renderer.handle_event(event)
    }

    pub fn render_world(&mut self, world: &mut GameWorld, render_data: RenderData) -> Result<(), SurfaceError>
    {
        self.gui_renderer.begin_frame();
        self.gui_renderer.draw_ui(|c| {
            world.on_gui(render_data.frame_state, c)
        });
        self.gui_renderer.end_frame();

        self.voxel_renderer.update(world.main_camera.camera(), &render_data.gpu_state.queue());
        
        let _ = self.voxel_renderer.get_profiling_info();
        
        self.renderer.render(&mut [&mut self.voxel_renderer, &mut self.gui_renderer])

    }
}