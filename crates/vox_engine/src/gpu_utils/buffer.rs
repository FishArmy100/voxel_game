use std::{marker::PhantomData, ops::RangeBounds};

use wgpu::util::DeviceExt;

use crate::utils::Byteable;

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

    pub fn enqueue_write(&mut self, data: &[T], queue: &wgpu::Queue)
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

pub trait VertexData : Byteable
{
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub struct VertexBuffer<T> where T : VertexData
{
    buffer: GBuffer<T>
}

impl<T> VertexBuffer<T> where T : VertexData
{
    pub fn buffer(&self) -> &GBuffer<T> { &self.buffer }
    pub fn mut_buffer(&mut self) -> &GBuffer<T> { &mut self.buffer }

    pub fn length(&self) -> u64 { self.buffer.length() }
    pub fn capacity(&self) -> u64 { self.buffer.capacity() }
    pub fn size(&self) -> u64 { self.buffer.size() }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> { T::desc() }

    pub fn usage() -> wgpu::BufferUsages
    {
        wgpu::BufferUsages::VERTEX      | 
        wgpu::BufferUsages::COPY_DST    | 
        wgpu::BufferUsages::COPY_SRC
    }

    pub fn new(data: &[T], device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let buffer = GBuffer::new(data, Self::usage(), device, label);
        Self 
        { 
            buffer 
        }
    }

    pub fn with_capacity(capacity: u64, device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let buffer = GBuffer::<T>::with_capacity(capacity, Self::usage(), device, label);
        Self 
        { 
            buffer 
        }
    }

    pub fn slice(&self, start: u64, end: u64) -> wgpu::BufferSlice { self.buffer.slice(start, end) }
    pub fn slice_all(&self) -> wgpu::BufferSlice { self.buffer.slice_all() }
    pub fn as_entire_binding(&self) -> wgpu::BindingResource { self.buffer.as_entire_binding() }

    pub fn enqueue_write(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        self.buffer.enqueue_write(data, queue);
    }
}

pub struct IndexBuffer 
{
    buffer: wgpu::Buffer,
    capacity: u64
}

impl IndexBuffer
{
    pub fn capacity(&self) -> u64 { self.capacity }

    pub fn new(indices: &[u32], device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let capacity = indices.len() as u64;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn new_empty(capacity: u64, device: &wgpu::Device, label: Option<&str>) -> Self
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: &vec![0 as u8; capacity as usize * std::mem::size_of::<u32>()],
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        });

        Self { buffer, capacity }
    }

    pub fn enqueue_set_data<T>(&self, queue: &wgpu::Queue, indices: &[u32])
        where T : VertexData
    {
        assert!(indices.len() as u64 <= self.capacity, "Data is larger than the capacity of this buffer.");

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(indices));
    }

    pub fn slice<B>(&self, bounds: B) -> wgpu::BufferSlice
        where B : RangeBounds<wgpu::BufferAddress>
    {
        self.buffer.slice(bounds)
    }
}