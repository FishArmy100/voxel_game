mod octree;

use std::sync::Arc;
use crate::colors::Color;
use crate::math::{Vec3, Point3D};
use crate::rendering::voxel_render_stage::{VoxelFaceData, VoxelFaces, VoxelRenderData};

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

type VoxelArray<const S: usize> = [[[Voxel; S]; S]; S];

pub struct Chunk<const S: usize, const N: usize>
{
    data: VoxelArray<S>,
    position: Vec3<usize>,
    voxels: Arc<[VoxelData; N]>,
}

impl<const S: usize, const N: usize> Chunk<S, N>
{
    pub fn new<F>(generator: &F, position: Vec3<usize>, voxels: Arc<[VoxelData; N]>) -> Self 
        where F : Fn(usize, usize, usize) -> Voxel
    {
        let mut data = [[[Voxel::default(); S]; S]; S];
        for x in 0..S
        {
            for y in 0..S
            {
                for z in 0..S 
                {
                    data[x][y][z] = generator(x, y, z);
                }
            }
        }

        Self 
        {
            data,
            position,
            voxels,
        }
    }

    pub fn get_voxel_faces(&self) -> Vec<VoxelFaceData>
    {
        let mut faces = vec![];

        for x in 0..S
        {
            for y in 0..S
            {
                for z in 0..S 
                {
                    Self::add_faces(&self.data, self.voxels.as_slice(), x, y, z, self.position, &mut faces);
                }
            }
        }

        faces
    }

    fn has_face(data: &VoxelArray<S>, voxels: &[VoxelData], x: usize, y: usize, z: usize, face_id: u32) -> bool
    {
        match face_id
        {
            VoxelFaces::SOUTH => 
            {
                if z > S
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if z == S - 1
                {
                    true
                }
                else 
                {
                    let id = data[x][y][z + 1].id;
                    !voxels[id as usize].is_solid
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
                    let id = data[x][y][z - 1].id;
                    !voxels[id as usize].is_solid
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
                    let id = data[x - 1][y][z].id;
                    !voxels[id as usize].is_solid
                }
            },
            VoxelFaces::EAST => 
            {
                if x > S
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if x == S - 1
                {
                    true
                }
                else 
                {
                    let id = data[x + 1][y][z].id;
                    !voxels[id as usize].is_solid
                }
            },
            VoxelFaces::UP => 
            {
                if y > S
                {
                    panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z)
                }
                else if y == S - 1
                {
                    true
                }
                else 
                {
                    let id = data[x][y + 1][z].id;
                    !voxels[id as usize].is_solid
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
                    let id = data[x][y - 1][z].id;
                    !voxels[id as usize].is_solid
                }
            },
            _ => panic!("This should not be reached")
        }
    }

    fn add_faces(data: &VoxelArray<S>, voxels: &[VoxelData], x: usize, y: usize, z: usize, chunk_pos: Vec3<usize>, faces: &mut Vec<VoxelFaceData>)
    {
        if x >= S || y >= S || z >= S
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z);
        }

        let id = data[x][y][z].id;
        if !voxels[id as usize].is_solid
        {
            return;
        }

        let pos = chunk_pos.map(|v| v as u32) + Vec3::new(x as u32, y as u32, z as u32);

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::SOUTH)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::SOUTH);
            faces.push(face);
        }

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::NORTH)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::NORTH);
            faces.push(face);
        }

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::EAST)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::EAST);
            faces.push(face);
        }

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::WEST)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::WEST);
            faces.push(face);
        }

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::UP)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::UP);
            faces.push(face);
        }

        if Self::has_face(data, voxels, x, y, z, VoxelFaces::DOWN)
        {
            let face = VoxelFaceData::new(pos, id as u32, VoxelFaces::DOWN);
            faces.push(face);
        }
    }
}

pub struct VoxelTerrain<const S: usize, const N: usize>
{
    chunks: Vec<Chunk<S, N>>,
    position: Point3D<f32>,
    faces: Vec<VoxelFaceData>,
    voxel_types: Arc<[VoxelData; N]>
}

impl<const S: usize, const N: usize> VoxelTerrain<S, N>
{
    pub const fn chunk_size() -> usize {S}
    pub fn position(&self) -> Point3D<f32> {self.position}
    pub fn faces(&self) -> &[VoxelFaceData] {&self.faces}
    pub fn voxel_types(&self) -> &[VoxelData; N] {&self.voxel_types}

    pub fn new<F>(position: Point3D<f32>, size_in_chunks: Vec3<usize>, voxel_size: f32, voxel_types: Arc<[VoxelData; N]>, generator: &F) -> Self
        where F : Fn(Vec3<usize>) -> Voxel
    {
        let mut chunks = vec![];

        for chunk_x in 0..size_in_chunks.x
        {
            for chunk_y in 0..size_in_chunks.y
            {
                for chunk_z in 0..size_in_chunks.z
                {
                    let chunk_pos = Vec3::new(chunk_x * S, chunk_y * S, chunk_z * S);
                    let generator = |x, y, z| generator(Vec3::new(x + chunk_x * S, y + chunk_y * S, z + chunk_z * S));
                    
                    let chunk = Chunk::<S, N>::new(&generator, chunk_pos, voxel_types.clone());
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
            voxel_types
        }
    }
}