use std::{marker::PhantomData, sync::Arc};

use wgpu::util::{DeviceExt, BufferInitDescriptor};

pub mod compute;
pub mod bind_group;

pub struct GPUBuffer<T> where T : bytemuck::Pod + bytemuck::Zeroable
{
    device: Arc<wgpu::Device>,
    usage: wgpu::BufferUsages,
    buffer: wgpu::Buffer,
    length: u64,
    capacity: u64,
    phantom: PhantomData<T>,
    label: Option<String>
}

impl<T> GPUBuffer<T> where T : bytemuck::Pod + bytemuck::Zeroable
{
    pub fn as_entire_binding(&self) -> wgpu::BindingResource { self.buffer.as_entire_binding() }
    pub fn buffer(&self) -> &wgpu::Buffer { &self.buffer } // TODO: Temp, should replace
    pub fn len(&self) -> u64 { self.length }
    pub fn cap(&self) -> u64 { self.capacity }

    pub fn new(data: &[T], usage: wgpu::BufferUsages, label: Option<String>, device: Arc<wgpu::Device>) -> Self
    {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: match &label {
                Some(l) => Some(&l),
                None => None
            },
            contents: bytemuck::cast_slice(data),
            usage
        });

        Self 
        { 
            device,
            usage,
            buffer, 
            length: data.len() as u64, 
            capacity: data.len() as u64, 
            phantom: PhantomData {},
            label
        }
    }

    pub fn with_capacity(capacity: u64, usage: wgpu::BufferUsages, label: Option<String>, device: Arc<wgpu::Device>) -> Self
    {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: match &label {
                Some(l) => Some(&l),
                None => None
            },
            contents: &vec![0 as u8; (capacity * std::mem::size_of::<T>() as u64) as usize],
            usage
        });

        Self 
        { 
            device,
            usage,
            buffer, 
            length: 0, 
            capacity,
            phantom: PhantomData {},
            label
        }
    }

    pub fn enqueue_set_data(&mut self, data: &[T], queue: &wgpu::Queue)
    {
        if self.capacity < data.len() as u64
        {
            self.buffer = self.device.create_buffer_init(&BufferInitDescriptor { 
                label: match &self.label {
                    Some(l) => Some(&l),
                    None => None
                }, 
                contents: bytemuck::cast_slice(data), 
                usage: self.usage
            });

            self.length = data.len() as u64;
            self.capacity = data.len() as u64;
        }
        else 
        {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
            self.length = data.len() as u64;
        }
    }

    pub fn slice(&self, begin: u64, end: u64) -> wgpu::BufferSlice
    {
        assert!(begin <= end, "The begin index must be less than or equal to the end index.");
        assert!(end < self.capacity, "slice [{}..{}] is larger than the capacity of this buffer", begin, end);
        self.buffer.slice(begin..end)
    }

    pub fn slice_all(&self) -> wgpu::BufferSlice
    {
        self.buffer.slice(0..self.length)
    }

    pub fn read(&self) -> Vec<T>
    {
        let buffer_slice = self.slice_all();
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = pollster::block_on(receiver.receive())
        {
            let data = buffer_slice.get_mapped_range();
            let result = bytemuck::cast_slice(&data).to_vec();
            result
        } 
        else 
        {
            panic!("failed to run compute on gpu!")
        }
    }
}