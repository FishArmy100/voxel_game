use wgpu::BindGroupDescriptor;

use crate::{gpu_utils::GBuffer, utils::Byteable};

pub trait Entry
{
    fn get_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry;
    fn get_resource(&self) -> wgpu::BindingResource;
}

pub struct BindGroup
{
    layout: wgpu::BindGroupLayout,
    handle: wgpu::BindGroup
}

impl BindGroup
{
    pub fn layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
    pub fn bind_group(&self) -> &wgpu::BindGroup { &self.handle }

    pub fn new(entries: &[&dyn Entry], device: &wgpu::Device) -> Self
    {
        let mut entry_layouts = Vec::with_capacity(entries.len());
        for i in 0..entries.len()
        {
            entry_layouts.push(entries[i].get_layout(i as u32));
        }

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &entry_layouts 
        });

        let mut bind_group_entries = Vec::with_capacity(entries.len());
        for i in 0..entries.len()
        {
            bind_group_entries.push(wgpu::BindGroupEntry {
                resource: entries[i].get_resource(),
                binding: i as u32
            });
        }

        let handle = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &bind_group_entries
        });

        Self 
        { 
            layout, 
            handle 
        }
    }
}

pub struct Uniform<T> where T : Byteable
{
    buffer: GBuffer<T>, 
    visibility: wgpu::ShaderStages
}

impl<T> Uniform<T> where T : Byteable
{
    pub fn new(value: T, visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
    {
        let buffer_usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let buffer = GBuffer::new(&[value], buffer_usage, device, None);
        Self 
        { 
            buffer,
            visibility
        }
    }

    pub fn new_empty(visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self
    {
        let buffer_usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let buffer = GBuffer::<T>::with_capacity(1, buffer_usage, device, None);
        Self
        {
            buffer,
            visibility
        }
    }

    pub fn enqueue_write(&mut self, value: T, queue: &wgpu::Queue)
    {
        self.buffer.enqueue_write(&[value], queue);
    }
}

impl<T> Entry for Uniform<T> where T : Byteable
{
    fn get_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry 
    {
        wgpu::BindGroupLayoutEntry 
        { 
            binding, 
            visibility: self.visibility, 
            ty: wgpu::BindingType::Buffer 
            { 
                ty: wgpu::BufferBindingType::Uniform, 
                has_dynamic_offset: false, 
                min_binding_size: None 
            }, 
            count: None 
        }
    }

    fn get_resource(&self) -> wgpu::BindingResource 
    {
        self.buffer.as_entire_binding()
    }
}

pub struct Storage<T> where T : Byteable
{
    buffer: GBuffer<T>,
    visibility: wgpu::ShaderStages
}

impl<T> Storage<T> where T : Byteable
{
    pub fn buffer_usage(is_vertex: bool) -> wgpu::BufferUsages 
    {
        if is_vertex 
        {
            wgpu::BufferUsages::STORAGE |
            wgpu::BufferUsages::COPY_SRC
        }
        else 
        {
            wgpu::BufferUsages::COPY_DST | 
            wgpu::BufferUsages::COPY_SRC | 
            wgpu::BufferUsages::STORAGE
        }
    }

    pub fn new(data: &[T], visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
    {
        let is_vertex = visibility.contains(wgpu::ShaderStages::VERTEX);
        let buffer = GBuffer::new(data, Self::buffer_usage(is_vertex), device, None);

        Self 
        { 
            buffer, 
            visibility
        }
    }

    pub fn with_capacity(capacity: u64, visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
    {
        let is_vertex = visibility.contains(wgpu::ShaderStages::VERTEX);
        let buffer = GBuffer::<T>::with_capacity(capacity, Self::buffer_usage(is_vertex), device, None);

        Self 
        { 
            buffer, 
            visibility
        }
    }

    pub fn copy_to(&self, dest: &mut Storage<T>, command_encoder: &mut wgpu::CommandEncoder)
    {
        self.buffer.copy(&mut dest.buffer, command_encoder);
    }

    pub fn copy_to_mapped(&self, dest: &mut MappedBuffer<T>, command_encoder: &mut wgpu::CommandEncoder)
    {
        self.buffer.copy(&mut dest.buffer, command_encoder);
    }

    pub fn enqueue_write(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        self.buffer.enqueue_write(data, queue);
    }
}

impl<T> Entry for Storage<T> where T : Byteable
{
    fn get_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry 
    {
        let read_only = self.visibility.contains(wgpu::ShaderStages::VERTEX);

        wgpu::BindGroupLayoutEntry 
        { 
            binding, 
            visibility: self.visibility, 
            ty: wgpu::BindingType::Buffer 
            { 
                ty: wgpu::BufferBindingType::Storage 
                { 
                    read_only 
                }, 
                has_dynamic_offset: false, 
                min_binding_size: None 
            }, 
            count: None 
        }
    }

    fn get_resource(&self) -> wgpu::BindingResource 
    {
        self.buffer.as_entire_binding()
    }
}

pub struct MappedBuffer<T> where T : Byteable
{
    buffer: GBuffer<T>,
    visibility: wgpu::ShaderStages
}

impl<T> MappedBuffer<T> where T : Byteable
{
    pub fn buffer_usage() -> wgpu::BufferUsages 
    {
        wgpu::BufferUsages::COPY_DST | 
        wgpu::BufferUsages::MAP_READ
    }

    pub fn new(data: &[T], visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
    {
        let buffer = GBuffer::new(data, Self::buffer_usage(), device, None);

        Self 
        { 
            buffer, 
            visibility
        }
    }

    pub fn with_capacity(capacity: u64, visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self
    {
        let buffer = GBuffer::<T>::with_capacity(capacity, Self::buffer_usage(), device, None);

        Self 
        { 
            buffer, 
            visibility
        }
    }

    pub fn enqueue_write(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        self.buffer.enqueue_write(data, queue);
    }

    pub fn read(&self, device: &wgpu::Device) -> Vec<T>
    {
        self.buffer.read(device)
    }
}

impl<T> Entry for MappedBuffer<T> where T : Byteable
{
    fn get_layout(&self, binding: u32) -> wgpu::BindGroupLayoutEntry 
    {
        let read_only = self.visibility.contains(wgpu::ShaderStages::VERTEX);

        wgpu::BindGroupLayoutEntry 
        { 
            binding,
            visibility: self.visibility, 
            ty: wgpu::BindingType::Buffer 
            { 
                ty: wgpu::BufferBindingType::Storage 
                { 
                    read_only
                }, 
                has_dynamic_offset: false, 
                min_binding_size: None 
            }, 
            count: None 
        }
    }

    fn get_resource(&self) -> wgpu::BindingResource 
    {
        self.buffer.as_entire_binding()
    }
}