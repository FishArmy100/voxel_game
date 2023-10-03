use std::{marker::PhantomData, sync::{Mutex, Arc}, cell::RefCell};
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use crate::{utils::Byteable, math::Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPUVec3<T> where T : Byteable
{
    pub x: T,
    pub y: T,
    pub z: T
}

unsafe impl<T> bytemuck::Pod for GPUVec3<T> where T : Byteable {}
unsafe impl<T> bytemuck::Zeroable for GPUVec3<T> where T : Byteable {}

impl<T> GPUVec3<T> where T : Byteable 
{
    pub fn new(x: T, y: T, z: T) -> Self
    {
        Self 
        { 
            x, 
            y, 
            z 
        }
    }
}

impl<T> From<Vec3<T>> for GPUVec3<T> where T : Byteable
{
    fn from(value: Vec3<T>) -> Self 
    {
        GPUVec3::new(value.x, value.y, value.z)
    }
}

pub struct Buffer<T> where T : Byteable
{
    capacity: u64,
    length: u64,
    handle: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> where T : Byteable
{
    pub fn capacity(&self) -> u64 { self.capacity }
    pub fn length(&self) -> u64 { self.length }
    pub fn usage(&self) -> wgpu::BufferUsages { self.usage }

    pub fn new(data: &[T], usage: wgpu::BufferUsages, device: &wgpu::Device) -> Self
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
            _phantom: PhantomData {} 
        }
    }

    pub fn with_capacity(capacity: u64, usage: wgpu::BufferUsages, device: &wgpu::Device,) -> Self 
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

    pub fn enqueue_write(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        debug_assert!(data.len() as u64 <= self.capacity, "Cannot writing data with length {} to buffer with capacity {}", data.len(), self.capacity);
        
        queue.write_buffer(&self.handle, 0, bytemuck::cast_slice(data));
        self.length = data.len() as u64;
    }

    pub fn copy_to(&self, dest: &Buffer<T>, command_encoder: &mut wgpu::CommandEncoder)
    {
        debug_assert!(self.usage.contains(wgpu::BufferUsages::COPY_SRC) && dest.usage.contains(wgpu::BufferUsages::COPY_DST), "`self` buffer must be COPY_SRC and `dest` buffer must be COPY_DST");
        debug_assert!(self.length <= dest.capacity, "Cannot copy buffer with length {} to buffer with capacity {}", self.length, self.capacity);
        command_encoder.copy_buffer_to_buffer(&self.handle, 0, &dest.handle, 0, self.length);
    }

    pub fn read(&self, device: &wgpu::Device) -> Vec<T>
    {
        pollster::block_on(self.read_async(device))
    }

    pub async fn read_async(&self, device: &wgpu::Device) -> Vec<T>
    {
        debug_assert!(self.usage.contains(wgpu::BufferUsages::MAP_READ));
        let buffer_slice = self.handle.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        match receiver.receive().await
        {
            Some(Ok(())) => 
            {
                let data = buffer_slice.get_mapped_range();
                let result = bytemuck::cast_slice(&data).to_vec();
    
                drop(data);
                self.handle.unmap();
    
                result
            },
            Some(Err(error)) => 
            {
                panic!("{}", error);
            } 
            None => 
            {
                panic!("Failed to read data from buffer");
            }
        } 
    }
}

pub struct ShaderInfo<'a>
{
    pub entry_point: &'a str,
    pub source: &'a str
}