use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{Thread, JoinHandle, self};

use crate::rendering::VertexBuffer;
use crate::utils::Array3D;
use super::octree::Octree;
use super::{Voxel, VoxelData, VoxelFaceData};
use crate::rendering::voxel_render_stage::{VoxelFace};
use crate::math::{Vec3, Point3D};

pub struct Chunk
{
    data: Octree<Voxel>,
    chunk_index: Vec3<isize>,
    voxels: Arc<Vec<VoxelData>>,

    faces_buffer: VertexBuffer<VoxelFaceData>,
}

impl Chunk
{
    pub fn size(&self) -> usize { self.data.length() } 
    pub fn faces_buffer(&self) -> &VertexBuffer<VoxelFaceData> { &self.faces_buffer }
    pub fn octree(&self) -> &Octree<Voxel>
    {
        &self.data
    }

    pub fn new(generator: &dyn VoxelGenerator, chunk_index: Vec3<isize>, voxels: Arc<Vec<VoxelData>>, chunk_depth: usize, device: &wgpu::Device) -> Self
    {
        let mut data = Octree::new(chunk_depth);
        let chunk_position = chunk_index * data.length() as isize;

        for x in 0..data.length()
        {
            for y in 0..data.length()
            {
                for z in 0..data.length()
                {
                    let offset = Vec3::new(x, y, z).cast().unwrap();
                    data.insert([x, y, z].into(), generator.get(chunk_position + offset));
                }
            }
        }

        let faces = Self::get_voxel_faces(&data, data.length(), chunk_position);
        let faces_buffer = VertexBuffer::new(&faces, device, Some("Faces buffer"));

        Self 
        {
            data,
            chunk_index,
            voxels,
            faces_buffer
        }
    }

    fn get_voxel_faces(data: &Octree<Voxel>, size: usize, position: Vec3<isize>) -> Vec<VoxelFaceData>
    {
        let mut faces = vec![];

        for x in 0..size
        {
            for y in 0..size
            {
                for z in 0..size 
                {
                    Self::add_faces(data, size, Vec3::new(x, y, z), position, &mut faces);
                }
            }
        }

        faces
    }

    fn has_face(data: &Octree<Voxel>, size: usize, index: Vec3<usize>, face_id: VoxelFace) -> bool
    {
        match face_id
        {
            VoxelFace::South => 
            {
                if index.z > size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z)
                }
                else if index.z == size - 1
                {
                    true
                }
                else 
                {
                    data.get([index.x, index.y, index.z + 1].into()).is_none()
                }
            },
            VoxelFace::North => 
            {
                if index.z == 0
                {
                    true
                }
                else 
                {
                    data.get([index.x, index.y, index.z - 1].into()).is_none()
                }
            },
            VoxelFace::West => 
            {
                if index.x == 0
                {
                    true
                }
                else 
                {
                    data.get([index.x - 1, index.y, index.z].into()).is_none()
                }
            },
            VoxelFace::East => 
            {
                if index.x > size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z)
                }
                else if index.x == size - 1
                {
                    true
                }
                else 
                {
                    data.get([index.x + 1, index.y, index.z].into()).is_none()
                }
            },
            VoxelFace::Up => 
            {
                if index.y > size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z)
                }
                else if index.y == size - 1
                {
                    true
                }
                else 
                {
                    data.get([index.x, index.y + 1, index.z].into()).is_none()
                }
            },
            VoxelFace::Down => 
            {
                if index.y == 0
                {
                    true
                }
                else 
                {
                    data.get([index.x, index.y - 1, index.z].into()).is_none()
                }
            },
            _ => panic!("This should not be reached")
        }
    }

    fn add_faces(data: &Octree<Voxel>, size: usize, index: Vec3<usize>, chunk_pos: Vec3<isize>, faces: &mut Vec<VoxelFaceData>)
    {
        if index.x >= size || index.y >= size || index.z >= size
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z);
        }

        let Some(voxel) = data.get([index.x, index.y, index.z].into()) else { return; };

        let pos = chunk_pos.map(|v| v as i32) + Vec3::new(index.x as i32, index.y as i32, index.z as i32);

        if Self::has_face(data, size, index, VoxelFace::South)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::South.to_index());
            faces.push(face);
        }

        if Self::has_face(data, size, index, VoxelFace::North)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::North.to_index());
            faces.push(face);
        }

        if Self::has_face(data, size, index, VoxelFace::East)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::East.to_index());
            faces.push(face);
        }

        if Self::has_face(data, size, index, VoxelFace::West)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::West.to_index());
            faces.push(face);
        }

        if Self::has_face(data, size, index, VoxelFace::Up)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::Up.to_index());
            faces.push(face);
        }

        if Self::has_face(data, size, index, VoxelFace::Down)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::Down.to_index());
            faces.push(face);
        }
    }
}

pub trait VoxelGenerator : Send + Sync + 'static
{
    fn get(&self, index: Vec3<isize>) -> Option<Voxel>;
}

struct ChunkGenerator
{
    generator_func: Arc<dyn VoxelGenerator>,
    queue: VecDeque<Vec3<isize>>,
    thread: Option<JoinHandle<Chunk>>,

    device: Arc<wgpu::Device>,
    chunk_depth: usize,
    voxels: Arc<Vec<VoxelData>>
}

impl ChunkGenerator
{
    fn new(generator: Arc<dyn VoxelGenerator>, chunk_depth: usize, voxels: Arc<Vec<VoxelData>>, device: Arc<wgpu::Device>) -> Self
    {
        Self 
        { 
            generator_func: generator,
            queue: VecDeque::new(),
            thread: None,
            device,
            chunk_depth,
            voxels
        }
    }

    fn tick(&mut self) -> Option<Chunk>
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
            let generator = self.generator_func.clone();
            let chunk_index = front;
            let chunk_depth = self.chunk_depth;

            self.thread = Some(thread::spawn(move || {
                println!("starting to generate chunk {:?}", chunk_index);
                let chunk = Chunk::new(generator.as_ref(), chunk_index, voxels, chunk_depth, &device);
                println!("finished generating chunk {:?}", chunk_index);
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

pub struct VoxelTerrain
{
    info: TerrainInfo,
    chunks: Vec<Chunk>,
    device: Arc<wgpu::Device>,
    generator: ChunkGenerator
}

impl VoxelTerrain
{
    pub const fn chunk_size(&self) -> usize { (2 as usize).pow(self.info.chunk_depth as u32) }
    pub fn voxel_types(&self) -> &[VoxelData] { &self.info.voxel_types }
    pub fn chunks(&self) -> &[Chunk] { &self.chunks }
    pub fn info(&self) -> &TerrainInfo { &self.info }

    pub fn new(info: TerrainInfo, device: Arc<wgpu::Device>, generator: Arc<dyn VoxelGenerator>) -> Self
    {
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
        if self.chunks.iter().any(|c| c.chunk_index == chunk_index)
        {
            false
        }
        else 
        {
            self.generator.queue.push_back(chunk_index);
            true
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