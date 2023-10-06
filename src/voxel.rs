pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
pub mod terrain_renderer;
pub mod voxel_rendering;
use crate::colors::Color;
use crate::math::Vec3;
use crate::rendering::VertexData;
use crate::utils::Array3D;

use self::terrain_renderer::{VoxelMesh, VoxelFace, FaceDir};

const VOXEL_FACE_TRIANGLES: [u32; 6] = [2, 1, 0, 2, 3, 1];

pub trait VoxelStorage<T> : Sized where T : IVoxel
{
    fn new(depth: usize) -> Self;
    fn depth(&self) -> usize;
    fn get(&self, index: Vec3<usize>) -> Option<T>;
    fn insert(&mut self, index: Vec3<usize>, value: Option<T>);
    fn simplify(&mut self);
    fn is_empty(&self) -> bool;

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

    fn get_mesh(&self) -> VoxelMesh
    {
        get_voxel_faces(self)
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

fn get_voxel_faces<TStorage, TVoxel>(data: &TStorage) -> VoxelMesh
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let mut faces = VoxelMesh::new();

    let length = data.length();
    for x in 0..length
    {
        for y in 0..length
        {
            for z in 0..length 
            {
                add_faces(data, Vec3::new(x, y, z), &mut faces);
            }
        }
    }

    faces
}

fn has_face<TStorage, TVoxel>(data: &TStorage, index: Vec3<usize>, face_dir: FaceDir) -> bool
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let size = data.length();
    match face_dir
    {
        FaceDir::South => 
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
        FaceDir::North => 
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
        FaceDir::West => 
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
        FaceDir::East => 
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
        FaceDir::Up => 
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
        FaceDir::Down => 
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
    let pos = index.cast().unwrap();

    if has_face(data, index, FaceDir::South)
    {
        mesh.add_face(pos, FaceDir::South, voxel.id());
    }

    if has_face(data, index, FaceDir::North)
    {
        mesh.add_face(pos, FaceDir::North, voxel.id());
    }

    if has_face(data, index, FaceDir::East)
    {
        mesh.add_face(pos, FaceDir::East, voxel.id());
    }

    if has_face(data, index, FaceDir::West)
    {
        mesh.add_face(pos, FaceDir::West, voxel.id());
    }

    if has_face(data, index, FaceDir::Up)
    {
        mesh.add_face(pos, FaceDir::Up, voxel.id());
    }

    if has_face(data, index, FaceDir::Down)
    {
        mesh.add_face(pos, FaceDir::Down, voxel.id());
    }
}