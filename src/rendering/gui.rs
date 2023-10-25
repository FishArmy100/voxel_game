use winit::event_loop::EventLoop;

use super::{RenderStage, DrawCall};

pub struct GuiRenderer
{
    context: egui_winit::egui::Context, 
    platform: egui_winit::State,
    renderer: egui_wgpu::renderer::Renderer,

    render_stage: GuiRenderStage
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

pub struct GuiRenderStage
{

}

impl RenderStage for GuiRenderStage
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        todo!()
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        todo!()
    }
}

pub struct GuiDrawCall
{

}

impl DrawCall for GuiDrawCall
{
    fn bind_groups(&self) -> Box<[&crate::gpu_utils::BindGroup]> 
    {
        todo!()
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        todo!()
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
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