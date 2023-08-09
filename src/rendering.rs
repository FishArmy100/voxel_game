pub mod renderer;
pub mod voxel_render_stage;

use crate::math::{Vec3, Mat4x4, Point3D};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelUniform
{
    pub model: Mat4x4<f32>
}

impl ModelUniform
{
    pub fn new(mat: Mat4x4<f32>) -> Self
    {
        Self { model: mat }
    }

    pub fn from_position(position: Point3D<f32>) -> Self
    {
        let mat = Mat4x4::from_translation(Vec3::new(position.x, position.y, position.z));
        Self::new(mat)
    }
}

unsafe impl bytemuck::Pod for ModelUniform {}
unsafe impl bytemuck::Zeroable for ModelUniform {}

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