use std::{marker::PhantomData, sync::Arc};
use wgpu::util::DeviceExt;

use crate::utils::Byteable;

pub struct Buffer<T> where T : Byteable
{
    capacity: u64,
    length: u64,
    handle: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    device: Arc<wgpu::Device>,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> where T : Byteable
{
    pub fn new(data: &[T], usage: wgpu::BufferUsages, device: Arc<wgpu::Device>) -> Self
    {
        let handle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data),
            usage
        });

        let capacity = data.len() as u64;
        let length = capacity;

        Self 
        { 
            capacity, 
            length, 
            handle, 
            usage, 
            device, 
            _phantom: PhantomData {} 
        }
    }

    pub fn with_capacity(capacity: u64, usage: wgpu::BufferUsages, device: Arc<wgpu::Device>) -> Self 
    {
        let handle = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: capacity * std::mem::size_of::<T>() as u64,
            usage,
            mapped_at_creation: false,
        });

        Self 
        { 
            capacity, 
            length: 0, 
            handle, 
            usage, 
            device, 
            _phantom: PhantomData{} 
        }
    }

    pub fn get_slice(&self, start: u64, end: u64) -> wgpu::BufferSlice
    {
        debug_assert!(start < self.length && end < self.length, "Slice is not inside buffer");
        debug_assert!(start <= end, "`start` must be less that or equal to `end`");
        self.handle.slice(start..end)
    }

    pub fn slice_all(&self, start: u64, end: u64) -> wgpu::BufferSlice
    {
        self.handle.slice(0..self.length)
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource
    {
        self.handle.as_entire_binding()
    }
}