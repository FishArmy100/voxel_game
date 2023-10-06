use crate::math::Vec3;
use crate::rendering::{VertexData, VertexBuffer};
use crate::gpu_utils::{Uniform, Storage, BindGroup, GPUVec3, Entry};

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

    fn append_bytes(&self, bytes: &mut Vec<u8>) 
    {
        bytes.extend(bytemuck::cast_slice(&[*self]));
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

    pub fn to_render_data(&self, voxel_size: f32, position: Vec3<u32>, device: &wgpu::Device) -> VoxelRenderData
    {
        let vertex_buffer = VertexBuffer::new(&self.vertices, device, Some("Voxel Vertex Buffer"));
        let face_storage = Storage::new(&self.faces, wgpu::ShaderStages::VERTEX, device);
        let voxel_size_uniform = Uniform::new(voxel_size, wgpu::ShaderStages::VERTEX, device);
        let position_uniform = Uniform::new(position.into(), wgpu::ShaderStages::VERTEX, device);

        VoxelRenderData::new(vertex_buffer, face_storage, voxel_size_uniform, position_uniform, device)
    }
}

pub struct VoxelRenderData
{
    vertex_buffer: VertexBuffer<VoxelVertex>,
    face_storage: Storage<VoxelFace>,
    voxel_size_uniform: Uniform<f32>,
    position_uniform: Uniform<GPUVec3<u32>>,
    
    bind_group: BindGroup
}

impl VoxelRenderData
{
    pub fn bind_group(&self) -> &BindGroup { &self.bind_group }
    pub fn vertex_buffer(&self) -> &VertexBuffer<VoxelVertex> { &self.vertex_buffer }

    pub fn new(vertex_buffer: VertexBuffer<VoxelVertex>, face_storage: Storage<VoxelFace>, voxel_size_uniform: Uniform<f32>, position_uniform: Uniform<GPUVec3<u32>>, device: &wgpu::Device) -> Self
    {
        let bind_group = BindGroup::new(&[&face_storage, &voxel_size_uniform, &position_uniform], device);

        Self 
        { 
            vertex_buffer, 
            face_storage,
            voxel_size_uniform,
            position_uniform,
            bind_group
        }
    }

    pub fn construct_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        let face_storage_layout = Storage::<VoxelFace>::get_layout_static(wgpu::ShaderStages::VERTEX, 0);
        let voxel_size_layout = Uniform::<f32>::get_layout_static(wgpu::ShaderStages::VERTEX, 1);
        let position_layout = Uniform::<GPUVec3<u32>>::get_layout_static(wgpu::ShaderStages::VERTEX, 2);
        BindGroup::construct_layout_from_entries(&[face_storage_layout, voxel_size_layout, position_layout], device)
    }
}