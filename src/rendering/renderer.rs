use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::texture::Texture; 
use crate::colors::Color;

pub struct BindGroupData 
{
    name: String,
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl BindGroupData
{
    pub fn name(&self) -> &str { &self.name }
    pub fn layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
    pub fn buffer(&self) -> &wgpu::Buffer { &self.buffer }
    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }

    pub fn uniform<T>(name: String, data: T, shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        let layout = Self::get_uniform_layout(shader_stages, device);
        Self::uniform_with_layout(name, data, layout, device)
    }

    pub fn uniform_with_layout<T>(name: String, data: T, layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        let data_array = &[data];
        let data: &[u8] = bytemuck::cast_slice(data_array);

        let (buffer, bind_group) = Self::get_bind_group(&layout, data, device);
        Self { name, layout, buffer, bind_group }
    }

    pub fn get_uniform_layout(shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: shader_stages,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        })
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, data: T) 
        where T : bytemuck::Pod + bytemuck::Zeroable 
    {
        queue.write_buffer(self.buffer(), 0, bytemuck::cast_slice(&[data]));
    }

    fn get_bind_group(layout: &wgpu::BindGroupLayout, data: &[u8], device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroup)
    {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: None,
        });

        (buffer, bind_group)
    }
}

pub trait RenderStage
{
    fn bind_groups(&self) -> &[BindGroupData];
    fn render_pipeline(&self) -> &wgpu::RenderPipeline;
    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>;
}

pub trait DrawCall
{
    fn on_pre_draw(&self, queue: &wgpu::Queue);
    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>);
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

    fn render_stage(&self, stage: &dyn RenderStage, view: &wgpu::TextureView)
    {
        let bind_groups = stage.bind_groups();
        let pipeline = stage.render_pipeline();

        for draw_call in stage.get_draw_calls()
        {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command Encoder") });
            let mut render_pass = self.get_render_pass(&mut encoder, &view);

            draw_call.on_pre_draw(&self.queue);

            render_pass.set_pipeline(pipeline);
            for bind_group_index in 0..bind_groups.len()
            {
                render_pass.set_bind_group(bind_group_index as u32, &bind_groups[bind_group_index].bind_group, &[])
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