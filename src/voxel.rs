pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
use crate::colors::Color;
use crate::math::Vec3;
use crate::rendering::VertexData;
use crate::rendering::voxel_render_stage::VoxelRenderData;
use crate::utils::Array3D;

const VOXEL_FACE_TRIANGLES: [u32; 6] = [2, 1, 0, 2, 3, 1];

pub trait VoxelStorage<T> : Sized where T : IVoxel
{
    fn new(depth: usize) -> Self;
    fn depth(&self) -> usize;
    fn get(&self, index: Vec3<usize>) -> Option<T>;
    fn insert(&mut self, index: Vec3<usize>, value: Option<T>);
    fn simplify(&mut self);
    fn is_empty(&self) -> bool;

    fn get_faces(&self) -> VoxelMesh 
    {
        get_voxel_faces(self)
    }

    fn new_from_grid<TArg, TFunc>(depth: usize, grid: &Array3D<TArg>, mut sampler: TFunc) -> Self
        where TFunc : FnMut(&TArg) -> Option<T>
    {
        let mut storage = Self::new(depth);
        let length = storage.length();
        assert!(grid.width() == length && grid.height() == length && grid.depth() == length, "Array was not of the propper size.");

        for x in 0..length
        {
            for y in 0..length
            {
                for z in 0..length
                {
                    if let Some(voxel) = sampler(&grid[Vec3::new(x, y, z)])
                    {
                        storage.insert([x, y, z].into(), Some(voxel));
                    }
                }
            }
        }
        
        storage.simplify();
        storage
    }
}

pub trait VoxelStorageExt<T> where T : IVoxel
{
    fn length(&self) -> usize;
    fn voxel_count(&self) -> usize;
    fn insert_and_simplify(&mut self, index: Vec3<usize>, value: Option<T>);
}

impl<TStorage, TVoxel> VoxelStorageExt<TVoxel> for TStorage 
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    fn length(&self) -> usize {
        (2 as usize).pow(self.depth() as u32)
    }

    fn voxel_count(&self) -> usize {
        self.length().pow(3)
    }

    fn insert_and_simplify(&mut self, index: Vec3<usize>, value: Option<TVoxel>) {
        self.insert(index, value);
        self.simplify();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelData
{
    color: Color,
}

impl VoxelData
{
    pub fn new(color: Color) -> Self
    {
        Self { color }
    }

    pub fn get_render_data(&self) -> VoxelRenderData
    {
        VoxelRenderData::new(self.color)
    }
}

pub trait IVoxel : Clone + Eq
{
    fn id(&self) -> u16;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voxel 
{
    id: u16
}

impl Voxel
{
    pub fn new(id: u16) -> Self
    {
        Self { id }
    }
}

impl IVoxel for Voxel
{
    fn id(&self) -> u16 
    {
        self.id    
    }
}

#[cfg(test)]
mod test 
{
    use std::mem::size_of_val;

    use super::*;
    #[test]
    fn this_should_work()
    {
        let vertex = VoxelVertex {
            pos: [1, 0, 0].into(),
            face_id: 0,
            block_id: 0
        };

        let as_u64: u64 = bytemuck::cast(vertex);
        assert!(size_of_val(&vertex) == 8);
        assert!(as_u64 == 1);
    }

    pub fn get_byte(number: u64, offset: u32) -> u64
    {
        (number >> (offset * 8)) & 255
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoxelVertex
{
    pos: Vec3<u8>,
    face_id: u8,
    block_id: u32,
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]));
    }
}

pub struct VoxelMesh
{
    vertices: Vec<VoxelVertex>,
    triangles: Vec<u32>
}

impl VoxelMesh
{
    pub fn vertices(&self) -> &[VoxelVertex] { &self.vertices }
    pub fn triangles(&self) -> &[u32] { &self.triangles }

    pub fn new() -> Self 
    {
        Self 
        {
            vertices: vec![],
            triangles: vec![]
        }
    }

    pub fn add_face(&mut self, pos: Vec3<u8>, face_id: VoxelFace, block_id: u16)
    {
        let vertex = VoxelVertex {
            pos,
            face_id: face_id.to_index() as u8,
            block_id: block_id as u32
        };

        self.vertices.extend([vertex; 4]);
        self.triangles.extend(VOXEL_FACE_TRIANGLES.map(|i| i + 1))
    }
}

pub enum VoxelFace 
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl VoxelFace
{
    pub fn to_index(&self) -> u32
    {
        match self 
        {
            VoxelFace::Up => 0,
            VoxelFace::Down => 1,
            VoxelFace::North => 2,
            VoxelFace::South => 3,
            VoxelFace::East => 4,
            VoxelFace::West => 5,
        }
    }

    pub fn from_index(index: u32) -> Self
    {
        match index 
        {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::North,
            3 => Self::South,
            4 => Self::East,
            5 => Self::West,
            _ => panic!("Invalid index {}", index)
        }
    }
}

fn get_voxel_faces<TStorage, TVoxel>(data: &TStorage) -> VoxelMesh
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let mut mesh = VoxelMesh::new();

    let length = data.length();
    for x in 0..length
    {
        for y in 0..length
        {
            for z in 0..length 
            {
                add_faces(data, Vec3::new(x, y, z), &mut mesh);
            }
        }
    }

    mesh
}

fn has_face<TStorage, TVoxel>(data: &TStorage, index: Vec3<usize>, face_id: VoxelFace) -> bool
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let size = data.length();
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

fn add_faces<TStorage, TVoxel>(data: &TStorage, index: Vec3<usize>, mesh: &mut VoxelMesh)
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let size = data.length();

    if index.x >= size || index.y >= size || index.z >= size
    {
        panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z);
    }

    let Some(voxel) = data.get([index.x, index.y, index.z].into()) else { return; };
    let face_index = Vec3::new(index.x as u8, index.y as u8, index.z as u8);

    if has_face(data, index, VoxelFace::South)
    {
        mesh.add_face(face_index, VoxelFace::South, voxel.id())
    }

    if has_face(data, index, VoxelFace::North)
    {
        mesh.add_face(face_index, VoxelFace::North, voxel.id())
    }

    if has_face(data, index, VoxelFace::East)
    {
        mesh.add_face(face_index, VoxelFace::East, voxel.id())
    }

    if has_face(data, index, VoxelFace::West)
    {
        mesh.add_face(face_index, VoxelFace::West, voxel.id())
    }

    if has_face(data, index, VoxelFace::Up)
    {
        mesh.add_face(face_index, VoxelFace::Up, voxel.id())
    }

    if has_face(data, index, VoxelFace::Down)
    {
        mesh.add_face(face_index, VoxelFace::Down, voxel.id())
    }
}