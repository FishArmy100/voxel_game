pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
pub mod voxel_rendering;
use crate::colors::Color;
use crate::math::Vec3;
use crate::rendering::VertexData;
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