use std::sync::Arc;
use crate::math::Color;
use crate::gpu_utils::texture::Texture;

pub trait RenderStage
{
    fn on_draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, depth_texture: &Texture);
}

pub struct Renderer
{
    device: Arc<wgpu::Device>,
    surface: Arc<wgpu::Surface>,
    queue: Arc<wgpu::Queue>,
    depth_texture: Texture,
    clear_color: Color
}

impl Renderer
{
    pub fn new(device: Arc<wgpu::Device>, surface: Arc<wgpu::Surface>, queue: Arc<wgpu::Queue>, config: &wgpu::SurfaceConfiguration, clear_color: Color) -> Self
    {
        let depth_texture = Texture::create_depth_texture(&device, config, "depth_texture");
        Self 
        { 
            device, 
            surface, 
            queue, 
            depth_texture,
            clear_color
        }
    }

    pub fn render(&self, stages: &mut [&mut dyn RenderStage]) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.clear_color(self.clear_color, &view);

        for stage in stages.iter_mut()
        {
            stage.on_draw(&self.device, &self.queue, &view, &self.depth_texture);
        }

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration)
    {
        self.depth_texture = Texture::create_depth_texture(&self.device, config, "depth_texture");
    }

    fn clear_color(&self, clear_color: Color, view: &wgpu::TextureView)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("Render Encoder")
        });

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations{
                    load: wgpu::LoadOp::Clear(clear_color.to_wgpu()),
                    store: true,
                }
            })],

            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}