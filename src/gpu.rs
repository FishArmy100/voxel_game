use std::{marker::PhantomData, sync::Mutex, cell::RefCell};

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

impl<T> Into<GPUVec3<T>> for Vec3<T> where T : Byteable
{
    fn into(self) -> GPUVec3<T> 
    {
        GPUVec3::new(self.x, self.y, self.z)
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

    pub fn length(&self) -> u64 { self.length }
    pub fn capacity(&self) -> u64 { self.capacity }

    pub fn enqueue_set(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        self.length = data.len() as u64;
        queue.write_buffer(&self.handle, 0, bytemuck::cast_slice(data));
    }

    pub fn slice(&self, start: u64, end: u64) -> wgpu::BufferSlice
    {
        assert!(start <= end, "Start index must be less than or equel to the end index");
        assert!(end < self.length(), "Slice is larger than the contained data");
        self.handle.slice(start..end)
    }

    pub fn slice_all(&self) -> wgpu::BufferSlice
    {
        self.handle.slice(0..self.length())
    }

    pub async fn read_async(&self, device: &wgpu::Device) -> Vec<T>
    {
        let buffer_slice = self.handle.slice(..);
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
        
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource
    {
        self.handle.as_entire_binding()
    }
}