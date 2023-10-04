use std::sync::Arc;

use crate::{math::Vec3, rendering::{VertexData, VertexBuffer}, gpu_utils::{Storage, BindGroup, Uniform}, camera::{Camera, CameraUniform}};

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
    location: Vec3<u32>,
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
            location, 
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
    pub vertex_index: u32
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VoxelVertex
{
    pub fn new(face_index: u32, vertex_index: u32) -> Self 
    {
        Self 
        { 
            face_index, 
            vertex_index 
        }
    }
}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32];

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
    faces: Vec<VoxelFace>
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
            faces: vec![] 
        }
    }

    pub fn add_face(&mut self, location: Vec3<u32>, direction: FaceDir, voxel_id: u16)
    {
        let face_count = self.vertices.len() as u32;
        self.vertices.extend(
        [
            VoxelVertex::new(face_count, 0), 
            VoxelVertex::new(face_count, 1), 
            VoxelVertex::new(face_count, 2), 
            VoxelVertex::new(face_count, 2),
            VoxelVertex::new(face_count, 3),
            VoxelVertex::new(face_count, 0)
        ]);

        self.faces.push(VoxelFace::new(location, direction, voxel_id))
    }
}

pub struct VoxelRenderStage
{
    device: Arc<wgpu::Device>,

    camera_uniform: Uniform<CameraUniform>,
    face_storage: Storage<VoxelFace>,
    vertex_buffer: VertexBuffer<VoxelVertex>,

    bind_group: BindGroup,

    render_pipeline: wgpu::RenderPipeline,

    camera: Camera,
}

impl VoxelRenderStage
{
    pub fn new(mesh: VoxelMesh, camera: Camera, device: Arc<wgpu::Device>) -> Self 
    {
        let camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);
        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, &device);

        let face_storage = Storage::new(mesh.faces(), wgpu::ShaderStages::VERTEX, &device);
        let bind_group = BindGroup::new(&[&camera_uniform, &face_storage], &device);
    }
}