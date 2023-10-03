use wgpu::BindGroupDescriptor;

use crate::{gpu::Buffer, utils::Byteable};

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

    pub fn enqueue_set<'s, 'r>(&'s self, index: u32, render_pass: &'r mut wgpu::RenderPass<'r>)
        where 's : 'r
    {
        render_pass.set_bind_group(index, &self.handle, &[]);
    }

    pub fn compute_enqueue_set<'s, 'c>(&'s self, index: u32, compute_pass: &'c mut wgpu::ComputePass<'c>)
        where 's : 'c
    {
        compute_pass.set_bind_group(index, &self.handle, &[]);
    }
}

pub struct Uniform<T> where T : Byteable
{
    buffer: Buffer<T>, 
    visibility: wgpu::ShaderStages
}

impl<T> Uniform<T> where T : Byteable
{
    pub fn new(value: T, visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self 
    {
        let buffer_usage = wgpu::BufferUsages::UNIFORM;
        let buffer = Buffer::new(&[value], buffer_usage, device);
        Self 
        { 
            buffer,
            visibility
        }
    }

    pub fn new_empty(visibility: wgpu::ShaderStages, device: &wgpu::Device) -> Self
    {
        let buffer_usage = wgpu::BufferUsages::UNIFORM;
        let buffer = Buffer::<T>::with_capacity(1, buffer_usage, device);
        Self
        {
            buffer,
            visibility
        }
    }

    pub fn enqueue_set(&mut self, value: T, queue: &wgpu::Queue)
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