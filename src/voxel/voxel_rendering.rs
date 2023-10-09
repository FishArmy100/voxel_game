use crate::colors::Color;
use crate::math::Vec3;
use crate::gpu_utils::{Uniform, Storage, BindGroup, GPUVec3, Entry, VertexBuffer, VertexData};

pub enum FaceDir
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl FaceDir
{
    pub fn to_index(&self) -> u32
    {
        match self 
        {
            FaceDir::Up =>      0,
            FaceDir::Down =>    1,
            FaceDir::North =>   2,
            FaceDir::South =>   3,
            FaceDir::East =>    4,
            FaceDir::West =>    5,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelFace
{
    position: Vec3<u32>,
    direction: u32,
    voxel_id: u32,
}

unsafe impl bytemuck::Pod for VoxelFace {}
unsafe impl bytemuck::Zeroable for VoxelFace {}

impl VoxelFace
{
    pub fn new(position: Vec3<u32>, direction: FaceDir, voxel_id: u16) -> Self 
    {
        Self 
        { 
            position,
            direction: direction.to_index(), 
            voxel_id: voxel_id as u32 
        }
    }
}

impl VertexData for VoxelFace
{
    fn desc() -> wgpu::VertexBufferLayout<'static> 
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Uint32x3, 1 => Uint32, 2 => Uint32];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex 
{
    pub index: u32,
    pub color: Color
}

impl VoxelVertex
{
    pub const fn new(index: u32, color: Color) -> Self
    {
        Self { index, color }
    }
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static> 
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Uint32, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub const VOXEL_FACE_VERTICES: [VoxelVertex; 4] = [VoxelVertex::new(0, Color::WHITE), VoxelVertex::new(1, Color::RED), VoxelVertex::new(2, Color::GREEN), VoxelVertex::new(3, Color::BLUE)];
pub const VOXEL_FACE_TRIANGLES: [u16; 6] = [2, 1, 0, 2, 3, 1];

pub struct VoxelMesh
{
    faces: Vec<VoxelFace>,
}

impl VoxelMesh
{
    pub fn faces(&self) -> &Vec<VoxelFace> { &self.faces }

    pub fn new() -> Self 
    {
        Self 
        {
            faces: vec![],
        }
    }

    pub fn add_face(&mut self, location: Vec3<u32>, direction: FaceDir, voxel_id: u16)
    {
        self.faces.push(VoxelFace::new(location, direction, voxel_id))
    }

    pub fn create_buffers(&self, device: &wgpu::Device) -> VertexBuffer<VoxelFace>
    {
        VertexBuffer::new(&self.faces, device, Some("Face Instance Buffer"))
    }
}