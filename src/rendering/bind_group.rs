use std::marker::PhantomData;

pub trait Entry 
{
    fn get_layout() -> wgpu::BindGroupLayoutEntry;
}

pub struct Uniform<T>
{
    buffer: wgpu::Buffer,
    stages: wgpu::ShaderStages,
    _phantom: PhantomData<T>
}