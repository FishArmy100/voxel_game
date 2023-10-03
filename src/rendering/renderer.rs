use std::sync::Arc;
use crate::application::WindowSize;
use crate::texture::Texture; 
use crate::colors::Color;
use super::RenderStage;

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

    pub fn render(&self, stages: &[&dyn RenderStage]) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.clear_color(self.clear_color, &view);

        for stage in stages
        {
            self.render_stage(*stage, &view);
        }

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration)
    {
        self.depth_texture = Texture::create_depth_texture(&self.device, config, "depth_texture");
    }

    fn render_stage(&self, stage: &dyn RenderStage, view: &wgpu::TextureView)
    {
        let pipeline = stage.render_pipeline();

        for mut draw_call in stage.get_draw_calls()
        {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command Encoder") });
            let mut render_pass = self.get_render_pass(&mut encoder, &view);

            draw_call.on_pre_draw(&self.queue);
            let bind_groups = draw_call.bind_groups();

            render_pass.set_pipeline(pipeline);
            for bind_group_index in 0..bind_groups.len()
            {
                render_pass.set_bind_group(bind_group_index as u32, bind_groups[bind_group_index].bind_group(), &[]);
            }

            draw_call.on_draw(&mut render_pass);
            drop(render_pass);
            self.queue.submit(std::iter::once(encoder.finish())); 
        }
    }

    fn get_render_pass<'this: 'e, 'e, 't: 'e>(&'this self, encoder: &'e mut wgpu::CommandEncoder, view: &'t wgpu::TextureView) -> wgpu::RenderPass<'e>
    {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations{
                    load: wgpu::LoadOp::Load,
                    store: true,
                }
            })],

            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
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