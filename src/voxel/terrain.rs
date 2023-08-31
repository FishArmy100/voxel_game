use crate::utils::Array3D;
use super::octree::Octree;
use super::{Voxel, VoxelData, VoxelFaceData};
use crate::rendering::voxel_render_stage::{VoxelFace};
use crate::math::{Vec3, Point3D};

pub struct Chunk
{
    data: Octree<Voxel>,
    position: Vec3<usize>,
    voxels: Vec<VoxelData>
}

impl Chunk
{
    pub fn size(&self) -> usize {self.data.length()} 

    pub fn new<F>(generator: &F, position: Vec3<usize>, voxels: Vec<VoxelData>, chunk_depth: usize) -> Self 
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

        Self 
        {
            data,
            position,
            voxels
        }
    }

    pub fn get_voxel_faces(&self) -> Vec<VoxelFaceData>
    {
        let mut faces = vec![];

        for x in 0..self.size()
        {
            for y in 0..self.size()
            {
                for z in 0..self.size() 
                {
                    self.add_faces(x, y, z, self.position, &mut faces);
                }
            }
        }

        faces
    }

    fn has_face(&self, x: usize, y: usize, z: usize, face_id: VoxelFace) -> bool
    {
        match face_id
        {
            VoxelFace::South => 
            {
                if z > self.size()
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if z == self.size() - 1
                {
                    true
                }
                else 
                {
                    self.data.get([x, y, z + 1].into()).is_none()
                }
            },
            VoxelFace::North => 
            {
                if z == 0
                {
                    true
                }
                else 
                {
                    self.data.get([x, y, z - 1].into()).is_none()
                }
            },
            VoxelFace::West => 
            {
                if x == 0
                {
                    true
                }
                else 
                {
                    self.data.get([x - 1, y, z].into()).is_none()
                }
            },
            VoxelFace::East => 
            {
                if x > self.size()
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if x == self.size() - 1
                {
                    true
                }
                else 
                {
                    self.data.get([x + 1, y, z].into()).is_none()
                }
            },
            VoxelFace::Up => 
            {
                if y > self.size()
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if y == self.size() - 1
                {
                    true
                }
                else 
                {
                    self.data.get([x, y + 1, z].into()).is_none()
                }
            },
            VoxelFace::Down => 
            {
                if y == 0
                {
                    true
                }
                else 
                {
                    self.data.get([x, y - 1, z].into()).is_none()
                }
            },
            _ => panic!("This should not be reached")
        }
    }

    fn add_faces(&self, x: usize, y: usize, z: usize, chunk_pos: Vec3<usize>, faces: &mut Vec<VoxelFaceData>)
    {
        if x >= self.size() || y >= self.size() || z >= self.size()
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z);
        }

        let Some(voxel) = self.data.get([x, y, z].into()) else { return; };

        let pos = chunk_pos.map(|v| v as u32) + Vec3::new(x as u32, y as u32, z as u32);

        if self.has_face(x, y, z, VoxelFace::South)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::South.to_index());
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFace::North)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::North.to_index());
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFace::East)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::East.to_index());
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFace::West)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::West.to_index());
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFace::Up)
        {
            let face = VoxelFaceData::new(pos, voxel.id as u32, VoxelFace::Up.to_index());
            faces.push(face);
        }

        if self.has_face(x, y, z, VoxelFace::Down)
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
        where F : Fn(Vec3<usize>) -> Option<Voxel>
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