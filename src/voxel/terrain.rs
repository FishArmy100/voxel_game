use std::collections::VecDeque;
use std::ops::{RangeBounds, Range};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{Thread, JoinHandle, self};
use std::time::SystemTime;

use cgmath::Array;

use crate::gpu::ShaderInfo;
use crate::rendering::{VertexBuffer, IndexBuffer, BindGroupData};
use crate::utils::Array3D;
use crate::voxel::world_gen::VoxelGenerator;
use super::octree::Octree;
use super::{Voxel, VoxelData, VoxelFace, VoxelStorage, VoxelStorageExt, VoxelMesh};
use crate::rendering::voxel_render_stage::{VoxelFaceOrientation, VoxelVertex};
use crate::math::{Vec3, Point3D};

pub struct ChunkMeshData
{
    vertex_buffer: VertexBuffer<VoxelVertex>,
    index_buffer: IndexBuffer,
    faces_bind_group: BindGroupData
}

impl ChunkMeshData
{
    pub fn vertex_buffer(&self) -> &VertexBuffer<VoxelVertex> { &self.vertex_buffer }
    pub fn index_buffer(&self) -> &IndexBuffer { &self.index_buffer }
    pub fn faces_bind_group(&self) -> &BindGroupData { &self.faces_bind_group }

    pub fn new(mesh: VoxelMesh, device: &wgpu::Device) -> Self
    {
        Self 
        { 
            vertex_buffer: VertexBuffer::new(mesh.vertices(), device, "Voxel Vertex Buffer".into()), 
            index_buffer: IndexBuffer::new(&mesh.triangles, device, "Voxel Index Buffer".into()), 
            faces_bind_group: BindGroupData::storage("Faces Bind Group".into(), mesh.faces(), wgpu::ShaderStages::VERTEX, device) 
        }
    }
}

pub struct Chunk<TStorage> where TStorage : VoxelStorage<Voxel>
{
    data: TStorage,
    index: Vec3<isize>,
    voxels: Arc<Vec<VoxelData>>,

    mesh_data: Option<ChunkMeshData>,
}

impl<TStorage> Chunk<TStorage> where TStorage : VoxelStorage<Voxel>
{
    pub fn size(&self) -> usize { self.data.length() } 
    pub fn mesh_data(&self) -> &Option<ChunkMeshData> { &self.mesh_data }
    pub fn index(&self) -> Vec3<isize> { self.index }
    pub fn storage(&self) -> &TStorage { &self.data }

    pub fn new(mut generator: MutexGuard<VoxelGenerator>, index: Vec3<isize>, voxels: Arc<Vec<VoxelData>>, chunk_depth: usize, device: &wgpu::Device) -> Self
    {
        let length = (2 as isize).pow(chunk_depth as u32);
        let chunk_position = index * length;
        let voxel_grid = generator.run(index.cast().unwrap());
        
        let now = SystemTime::now();
        let data = TStorage::new_from_grid(chunk_depth, &voxel_grid, |i| {
            if *i > 0 
            {
                Some(Voxel::new(*i as u16))
            }
            else 
            {
                None
            }
        });
        
        let elapsed = now.elapsed().unwrap().as_micros() as f32 / 1000.0;
        println!("took {}ms to create and populate voxel storage", elapsed);

        let mesh_data = if data.is_empty()
        {
            None
        }
        else 
        {    
            let now = SystemTime::now();
            let mesh = data.get_mesh();
            let elapsed = now.elapsed().unwrap().as_micros() as f32 / 1000.0;
            println!("took {}ms to generated the faces", elapsed);
            
            Some(ChunkMeshData::new(mesh, device))
        };

        Self 
        {
            data,
            index,
            voxels,
            mesh_data
        }
    }
}

struct ChunkGenerator<TStorage> where TStorage : VoxelStorage<Voxel>
{
    generator: Arc<Mutex<VoxelGenerator>>,
    queue: VecDeque<Vec3<isize>>,
    thread: Option<JoinHandle<Chunk<TStorage>>>,

    device: Arc<wgpu::Device>,
    chunk_depth: usize,
    voxels: Arc<Vec<VoxelData>>
}

impl<TStorage> ChunkGenerator<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn new(generator: VoxelGenerator, chunk_depth: usize, voxels: Arc<Vec<VoxelData>>, device: Arc<wgpu::Device>) -> Self
    {
        Self 
        { 
            generator: Arc::new(Mutex::new(generator)),
            queue: VecDeque::new(),
            thread: None,
            device,
            chunk_depth,
            voxels
        }
    }

    fn tick(&mut self) -> Option<Chunk<TStorage>>
    {
        let mut chunk = None;
        if self.thread.is_some() && self.thread.as_ref().unwrap().is_finished()
        {
            let Some(thread) = std::mem::replace(&mut self.thread, None) else {
                panic!("This should not have been called")
            };

            chunk = Some(thread.join().unwrap());
        }

        if self.thread.is_some() { return None; }

        self.thread = None;

        if let Some(front) = self.queue.pop_front()
        {
            let device = self.device.clone();
            let voxels = self.voxels.clone();
            let generator = self.generator.clone();
            let chunk_index = front;
            let chunk_depth = self.chunk_depth;

            self.thread = Some(thread::spawn(move || {
                let mutex = generator.lock().unwrap();
                let chunk = Chunk::new(mutex, chunk_index, voxels, chunk_depth, &device);
                chunk
            }))
        }

        chunk
    }
}

pub struct TerrainInfo
{
    pub chunk_depth: usize,
    pub voxel_size: f32,
    pub voxel_types: Arc<Vec<VoxelData>>
}

pub struct VoxelTerrain<TStorage> where TStorage : VoxelStorage<Voxel>
{
    info: TerrainInfo,
    chunks: Vec<Chunk<TStorage>>,
    device: Arc<wgpu::Device>,
    generator: ChunkGenerator<TStorage>
}

impl<TStorage> VoxelTerrain<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    pub const fn chunk_size(&self) -> usize { (2 as usize).pow(self.info.chunk_depth as u32) }
    pub fn voxel_types(&self) -> &[VoxelData] { &self.info.voxel_types }
    pub fn chunks(&self) -> &[Chunk<TStorage>] { &self.chunks }
    pub fn info(&self) -> &TerrainInfo { &self.info }

    pub fn new(info: TerrainInfo, shader_info: ShaderInfo, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self
    {
        let chunk_size = Vec3::from_value((2 as u32).pow(info.chunk_depth as u32));

        let generator = VoxelGenerator::new(chunk_size, device.clone(), queue, shader_info);
        let voxel_types = info.voxel_types.clone();
        let chunk_depth = info.chunk_depth;
        Self 
        { 
            info, 
            chunks: vec![], 
            device: device.clone(), 
            generator: ChunkGenerator::new(generator, chunk_depth, voxel_types, device)
        }
    }

    pub fn generate_chunk(&mut self, chunk_index: Vec3<isize>) -> bool
    {
        if self.chunks.iter().any(|c| c.index == chunk_index)
        {
            false
        }
        else 
        {
            self.generator.queue.push_back(chunk_index);
            true
        }
    }

    pub fn generate_chunks<B>(&mut self, bounds: [B; 3]) where B : RangeBounds<isize> + IntoIterator<Item = isize> + Clone
    {
        for x in bounds[0].clone()
        {
            for y in bounds[1].clone()
            {
                for z in bounds[2].clone()
                {
                    self.generate_chunk(Vec3::new(x, y, z));
                }
            }
        }
    } 

    pub fn tick(&mut self)
    {
        if let Some(chunk) = self.generator.tick()
        {
            self.chunks.push(chunk);
        }
    }
}