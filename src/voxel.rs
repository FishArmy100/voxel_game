mod octree;

use std::sync::Arc;
use crate::colors::Color;
use crate::math::{Vec3, Point3D};
use crate::rendering::voxel_render_stage::{VoxelFaceData, VoxelFaces, VoxelRenderData};
use crate::utils::Array3D;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelData
{
    color: Color,
    is_solid: bool
}

impl VoxelData
{
    pub fn new(color: Color, is_solid: bool) -> Self
    {
        Self { color, is_solid }
    }

    pub fn get_render_data(&self) -> VoxelRenderData
    {
        VoxelRenderData::new(self.color)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Voxel 
{
    id: u8
}

impl Voxel
{
    pub fn new(id: u8) -> Self
    {
        Self { id }
    }
}

pub struct Chunk
{
    data: Array3D<Voxel>,
    position: Vec3<usize>,
    voxels: Vec<VoxelData>,
    size: usize
}

impl Chunk
{
    pub fn new<F>(generator: &F, position: Vec3<usize>, voxels: Vec<VoxelData>, size: usize) -> Self 
        where F : Fn(usize, usize, usize) -> Voxel
    {
        let mut data = Array3D::new(size, size, size, generator);

        Self 
        {
            data,
            position,
            voxels,
            size
        }
    }

    pub fn get_voxel_faces(&self) -> Vec<VoxelFaceData>
    {
        let mut faces = vec![];

        for x in 0..self.size
        {
            for y in 0..self.size
            {
                for z in 0..self.size 
                {
                    self.add_faces(x, y, z, self.position, &mut faces);
                }
            }
        }

        faces
    }

    fn has_face(&self, x: usize, y: usize, z: usize, face_id: u32) -> bool
    {
        match face_id
        {
            VoxelFaces::SOUTH => 
            {
                if z > self.size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if z == self.size - 1
                {
                    true
                }
                else 
                {
                    let id = self.data[(x, y, z + 1)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            VoxelFaces::NORTH => 
            {
                if z == 0
                {
                    true
                }
                else 
                {
                    let id = self.data[(x, y, z - 1)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            VoxelFaces::WEST => 
            {
                if x == 0
                {
                    true
                }
                else 
                {
                    let id = self.data[(x - 1, y, z)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            VoxelFaces::EAST => 
            {
                if x > self.size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if x == self.size - 1
                {
                    true
                }
                else 
                {
                    let id = self.data[(x + 1, y, z)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            VoxelFaces::UP => 
            {
                if y > self.size
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if y == self.size - 1
                {
                    true
                }
                else 
                {
                    let id = self.data[(x, y + 1, z)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            VoxelFaces::DOWN => 
            {
                if y == 0
                {
                    true
                }
                else 
                {
                    let id = self.data[(x, y - 1, z)].id;
                    !self.voxels[id as usize].is_solid
                }
            },
            _ => panic!("This should not be reached")
        }
    }

    fn add_faces(&self, x: usize, y: usize, z: usize, chunk_pos: Vec3<usize>, faces: &mut Vec<VoxelFaceData>)
    {
        if x >= self.size || y >= self.size || z >= self.size
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z);
        }

        let id = self.data[(x, y, z)].id;
        if !self.voxels[id as usize].is_solid
        {
            return;
        }

        let pos = chunk_pos.map(|v| v as u32) + Vec3::new(x as u32, y as u32, z as u32);

        if self.has_face(x, y, z, VoxelFaces::SOUTH)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::SOUTH);
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFaces::NORTH)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::NORTH);
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFaces::EAST)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::EAST);
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFaces::WEST)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::WEST);
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFaces::UP)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::UP);
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFaces::DOWN)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::DOWN);
            faces.push(face);
        }
    }
}

pub struct VoxelTerrain
{
    chunks: Vec<Chunk>,
    position: Point3D<f32>,
    faces: Vec<VoxelFaceData>,
    voxel_types: Vec<VoxelData>,
    chunk_size: usize
}

impl VoxelTerrain
{
    pub const fn chunk_size(&self) -> usize { self.chunk_size }
    pub fn position(&self) -> Point3D<f32> { self.position }
    pub fn faces(&self) -> &[VoxelFaceData] { &self.faces }
    pub fn voxel_types(&self) -> &[VoxelData] { &self.voxel_types }

    pub fn new<F>(position: Point3D<f32>, size_in_chunks: Vec3<usize>, chunk_size: usize, voxel_size: f32, voxel_types: Vec<VoxelData>, generator: &F) -> Self
        where F : Fn(Vec3<usize>) -> Voxel
    {
        let mut chunks = vec![];

        for chunk_x in 0..size_in_chunks.x
        {
            for chunk_y in 0..size_in_chunks.y
            {
                for chunk_z in 0..size_in_chunks.z
                {
                    let chunk_pos = Vec3::new(chunk_x * chunk_size, chunk_y * chunk_size, chunk_z * chunk_size);
                    let generator = |x, y, z| generator(Vec3::new(x + chunk_x * chunk_size, y + chunk_y * chunk_size, z + chunk_z * chunk_size));
                    
                    let chunk = Chunk::new(&generator, chunk_pos, voxel_types.clone(), chunk_size);
                    chunks.push(chunk);
                }
            }
        }

        let mut faces = vec![];

        for chunk in &chunks
        {
            faces.extend(chunk.get_voxel_faces());
        }

        Self 
        { 
            chunks, 
            position,
            faces,
            voxel_types,
            chunk_size
        }
    }
}