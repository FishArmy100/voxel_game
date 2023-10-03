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

pub struct GBuffer<T> where T : Byteable
{
    length: u64,
    capacity: u64,
    handle: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    phantom: PhantomData<T>,
}

impl<T> GBuffer<T> where T : Byteable 
{
    pub fn new(data: &[T], usage: wgpu::BufferUsages, device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let length = data.len() as u64;
        let capacity = data.len() as u64;
        let handle = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &bytemuck::cast_slice(data),
            usage,
        });

        Self
        {
            length,
            capacity,
            handle,
            usage,
            phantom: PhantomData {}
        }
    }

    pub fn with_capacity(capacity: u64, usage: wgpu::BufferUsages, device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let handle = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: capacity * std::mem::size_of::<T>() as u64,
            usage,
            mapped_at_creation: false
        });

        Self
        {
            length: capacity,
            capacity,
            handle,
            usage,
            phantom: PhantomData {}
        }
    }

    pub fn length(&self) -> u64 { self.length }
    pub fn capacity(&self) -> u64 { self.capacity }
    pub fn size(&self) -> u64 { self.length() * std::mem::size_of::<T>() as u64 }

    pub fn enqueue_set(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        self.length = data.len() as u64;
        queue.write_buffer(&self.handle, 0, bytemuck::cast_slice(data));
    }

    pub fn slice(&self, start: u64, end: u64) -> wgpu::BufferSlice
    {
        assert!(start <= end, "Start index must be less than or equal to the end index");
        assert!(end < self.length(), "Slice is larger than the contained data");
        self.handle.slice((start * std::mem::size_of::<T>() as u64)..(end * std::mem::size_of::<T>() as u64))
    }

    pub fn slice_all(&self) -> wgpu::BufferSlice
    {
        self.handle.slice(0..self.size())
    }

    pub fn read(&self, device: &wgpu::Device) -> Vec<T>
    {
        pollster::block_on(self.read_async(device))
    }

    pub async fn read_async(&self, device: &wgpu::Device) -> Vec<T>
    {
        let buffer_slice = self.slice_all();
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        match receiver.receive().await
        {
            Some(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let result = bytemuck::cast_slice(&data).to_vec();
    
                drop(data);
                self.handle.unmap();
    
                result
            },
            Some(Err(error)) => {
                panic!("{}", error);
            } 
            None => {
                panic!("Failed to read data from buffer");
            }
        } 
    }

    pub fn copy(&self, dest: &mut GBuffer<T>, command_encoder: &mut wgpu::CommandEncoder)
    {
        assert!(dest.capacity >= self.length(), "Destination buffer capacity not large enough");
        command_encoder.copy_buffer_to_buffer(&self.handle, 0, &dest.handle, 0, self.size());
        dest.length = self.length;
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource
    {
        self.handle.as_entire_binding()
    }
}

pub struct ShaderInfo<'a>
{
    pub entry_point: &'a str,
    pub source: &'a str
}