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
    pos_x: u32,
    pos_y: u32,
    pos_z: u32,
    direction: u32,
    voxel_id: u32,
}

unsafe impl bytemuck::Pod for VoxelFace {}
unsafe impl bytemuck::Zeroable for VoxelFace {}

impl VoxelFace
{
    pub fn new(location: Vec3<u32>, direction: FaceDir, voxel_id: u16) -> Self 
    {
        Self 
        { 
            pos_x: location.x, 
            pos_y: location.y, 
            pos_z: location.z, 
            direction: direction.to_index(), 
            voxel_id: voxel_id as u32 
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex
{
    pub face_index: u32,
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VoxelVertex
{
    pub fn new(face_index: u32) -> Self 
    {
        Self 
        { 
            face_index, 
        }
    }
}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] =
            wgpu::vertex_attr_array![0 => Uint32];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub struct VoxelMesh
{
    vertices: Vec<VoxelVertex>,
    faces: Vec<VoxelFace>,
}

impl VoxelMesh
{
    pub fn faces(&self) -> &Vec<VoxelFace> { &self.faces }
    pub fn vertices(&self) -> &Vec<VoxelVertex> { &self.vertices }

    pub fn new() -> Self 
    {
        Self 
        { 
            vertices: vec![], 
            faces: vec![],
        }
    }

    pub fn add_face(&mut self, location: Vec3<u32>, direction: FaceDir, voxel_id: u16)
    {
        let face_count = self.faces.len() as u32;
        self.vertices.extend(
        [
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count)
        ]);

        self.faces.push(VoxelFace::new(location, direction, voxel_id))
    }

    pub fn create_buffers(&self, device: &wgpu::Device) -> (VertexBuffer<VoxelVertex>, Storage<VoxelFace>)
    {
        let vertex_buffer = VertexBuffer::new(&self.vertices, device, Some("Voxel Vertex Buffer"));
        let face_storage = Storage::new(&self.faces, wgpu::ShaderStages::VERTEX, device);
        (vertex_buffer, face_storage)
    }
}