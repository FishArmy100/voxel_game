use winit::event_loop::EventLoop;

use crate::rendering::RenderStage;

pub struct GuiRenderer
{
    context: egui_winit::egui::Context, 
    platform: egui_winit::State,
    renderer: egui_wgpu::renderer::Renderer,
}

impl GuiRenderer
{
    pub fn new<T>(desc: GuiRendererDescriptor<T>) -> Self
        where T : 'static
    {
        let GuiRendererDescriptor
        {
            event_loop,
            device,
            rt_format
        } = desc;

        let platform = egui_winit::State::new(event_loop);
        
        todo!()
    }
}

impl RenderStage for GuiRenderer
{
    fn on_draw(&self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, _depth_texture: &crate::gpu_utils::Texture) 
    {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GuiRendererDescriptor<'a, T> where T : 'static
{
    pub event_loop: &'a EventLoop<T>,
    pub device: &'a wgpu::Device,
    pub rt_format: wgpu::TextureFormat,
}