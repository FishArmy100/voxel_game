use std::{marker::PhantomData, sync::Arc};
use std::cell::{Ref, RefCell};
use wgpu::{util::DeviceExt, BindGroupDescriptor};

use super::GPUBuffer;

pub trait IBindGroup
{
    fn name(&self) -> &str;
    fn layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group_handle(&self) -> &wgpu::BindGroup;
}

pub trait UniformData
{
    const SIZE: usize;
    fn as_bytes(&self) -> &[u8];
}

impl<T> UniformData for T where T : bytemuck::Pod + bytemuck::Zeroable
{
    const SIZE: usize = std::mem::size_of::<Self>();
    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

pub struct UniformBindGroup<T> where T : UniformData
{
    name: String,
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    phantom: PhantomData<T>
}

impl<T> UniformBindGroup<T> where T : UniformData
{
    pub fn new(name: String, data: Option<T>, shader_stages: wgpu::ShaderStages, device: &wgpu::Device) -> Self
    {
        let layout = Self::get_uniform_layout(shader_stages, device);
        Self::with_layout(name, data, layout, device)
    }

    pub fn with_layout(name: String, data: Option<T>, layout: wgpu::BindGroupLayout, device: &wgpu::Device) -> Self
    {
        let empty = vec![0 as u8; T::SIZE];
        let bytes = match &data {
            Some(d) => d.as_bytes(),
            None => &empty,
        };

        let (buffer, bind_group) = Self::get_bind_group(&layout, &bytes, device);
        Self { name, layout, buffer, bind_group, phantom: PhantomData {} }
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

    pub fn enqueue_set_data(&self, queue: &wgpu::Queue, data: T)
    {
        queue.write_buffer(&self.buffer, 0, &data.as_bytes());
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

impl<T> IBindGroup for UniformBindGroup<T> where T : UniformData
{
    fn name(&self) -> &str { &self.name }
    fn layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
    fn bind_group_handle(&self) -> &wgpu::BindGroup { &self.bind_group }
}

pub struct StorageBindGroup<T> where T : bytemuck::Zeroable + bytemuck::Pod
{
    device: Arc<wgpu::Device>,
    name: String,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    buffer: GPUBuffer<T>,
    phantom: PhantomData<T>,
}

impl<T> StorageBindGroup<T> where T : bytemuck::Zeroable + bytemuck::Pod
{
    pub fn buffer(&self) -> &GPUBuffer<T> { &self.buffer } // TODO: Temp, should be replaced
    pub fn enqueue_set_data(&mut self, data: &[T], queue: &wgpu::Queue) 
    { 
        self.buffer.enqueue_set_data(data, queue);
        self.bind_group = Self::get_bind_group(&self.device, &self.layout, &self.buffer); // makes sure the bind group is pointing to the right resource
    }
    pub fn read(&self) -> Vec<T> { self.buffer.read() }

    pub fn new(data: &[T], shader_stages: wgpu::ShaderStages, name: String, device: Arc<wgpu::Device>) -> Self
    {
        let layout = Self::get_layout(&device, shader_stages);

        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC;
        let buffer = GPUBuffer::new(data, usage, None, device.clone());

        let bind_group = Self::get_bind_group(&device, &layout, &buffer);

        Self 
        { 
            device,
            name, 
            layout, 
            bind_group, 
            buffer, 
            phantom: PhantomData {}
        }
    }

    fn get_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &GPUBuffer<T>) -> wgpu::BindGroup
    {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ]
        });

        bind_group
    }

    fn get_layout(device: &wgpu::Device, shader_stages: wgpu::ShaderStages) -> wgpu::BindGroupLayout
    {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: shader_stages,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage 
                        { 
                            read_only: false 
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: None,
        })
    }
}

impl<T> IBindGroup for StorageBindGroup<T> where T : bytemuck::Pod + bytemuck::Zeroable
{
    fn name(&self) -> &str { &self.name }
    fn layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
    fn bind_group_handle(&self) -> &wgpu::BindGroup { &self.bind_group }
}