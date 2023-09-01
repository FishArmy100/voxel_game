use std::sync::Arc;

use crate::rendering::VertexBuffer;
use crate::utils::Array3D;
use super::octree::Octree;
use super::{Voxel, VoxelData, VoxelFaceData};
use crate::rendering::voxel_render_stage::{VoxelFace};
use crate::math::{Vec3, Point3D};

pub struct Chunk
{
    data: Octree<Voxel>,
    position: Vec3<usize>,
    voxels: Vec<VoxelData>,

    faces_buffer: VertexBuffer<VoxelFaceData>,
}

impl Chunk
{
    pub fn size(&self) -> usize {self.data.length()} 
    pub fn faces_buffer(&self) -> &VertexBuffer<VoxelFaceData> { &self.faces_buffer }

    pub fn new<F>(generator: &F, position: Vec3<usize>, voxels: Vec<VoxelData>, chunk_depth: usize, device: &wgpu::Device) -> Self 
        where F : Fn(usize, usize, usize) -> Option<Voxel>
    {
        let mut data = Octree::new(chunk_depth);
        for x in 0..data.length()
        {
            for y in 0..data.length()
            {
                for z in 0..data.length()
                {
                    data.insert([x, y, z].into(), generator(x, y, z))
                }
            }
        }

        let faces = Self::get_voxel_faces(&data, data.length(), position);
        let faces_buffer = VertexBuffer::new(&faces, device, Some("Faces buffer"));

        Self 
        {
            data,
            position,
            voxels,
            faces_buffer
        }
    }

    fn get_voxel_faces(data: &Octree<Voxel>, size: usize, position: Vec3<usize>) -> Vec<VoxelFaceData>
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

    fn add_faces(data: &Octree<Voxel>, size: usize, index: Vec3<usize>, chunk_pos: Vec3<usize>, faces: &mut Vec<VoxelFaceData>)
    {
        if index.x >= size || index.y >= size || index.z >= size
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z);
        }

        let Some(voxel) = data.get([index.x, index.y, index.z].into()) else { return; };

        let pos = chunk_pos.map(|v| v as u32) + Vec3::new(index.x as u32, index.y as u32, index.z as u32);

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

pub struct VoxelTerrain
{
    chunks: Vec<Chunk>,
    position: Point3D<f32>,
    voxel_types: Vec<VoxelData>,
    chunk_size: usize,
    device: Arc<wgpu::Device>
}

impl VoxelTerrain
{
    pub const fn chunk_size(&self) -> usize { self.chunk_size }
    pub fn position(&self) -> Point3D<f32> { self.position }
    pub fn voxel_types(&self) -> &[VoxelData] { &self.voxel_types }
    pub fn chunks(&self) -> &[Chunk] { &self.chunks }

    pub fn new<F>(position: Point3D<f32>, size_in_chunks: Vec3<usize>, chunk_depth: usize, voxel_size: f32, voxel_types: Vec<VoxelData>, device: Arc<wgpu::Device>, generator: &F) -> Self
        where F : Fn(Vec3<usize>) -> Option<Voxel>
    {
        let mut chunks = vec![];
        let mut current_chunk = 0;
        let chunk_size = (2 as usize).pow(chunk_depth as u32);

        for chunk_x in 0..size_in_chunks.x
        {
            for chunk_y in 0..size_in_chunks.y
            {
                for chunk_z in 0..size_in_chunks.z
                {
                    let chunk_pos = Vec3::new(chunk_x * chunk_size, chunk_y * chunk_size, chunk_z * chunk_size);
                    let generator = |x, y, z| generator(Vec3::new(x + chunk_x * chunk_size, y + chunk_y * chunk_size, z + chunk_z * chunk_size));
                    
                    let chunk = Chunk::new(&generator, chunk_pos, voxel_types.clone(), chunk_depth, &device);
                    chunks.push(chunk);

                    current_chunk += 1;
                    println!("Generated chunk {}/{}", current_chunk, size_in_chunks.x * size_in_chunks.y * size_in_chunks.z);
                }
            }
        }

        Self 
        { 
            chunks, 
            position,
            voxel_types,
            chunk_size,
            device
        }
    }
}