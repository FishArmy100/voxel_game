pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
pub mod terrain_renderer;
pub mod voxel_rendering;

use crate::math::{Vec3, Color};
use crate::utils::Array3D;

use self::voxel_rendering::{VoxelMesh, FaceDir};
pub use block_mesh::VoxelVisibility as Visibility;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VoxelIndex(u16);

impl From<u16> for VoxelIndex
{
    fn from(value: u16) -> Self 
    {
        VoxelIndex(value)
    }
}

impl From<VoxelIndex> for u16
{
    fn from(value: VoxelIndex) -> Self 
    {
        value.0
    }
}

#[derive(Debug)]
pub struct Voxel 
{
    pub color: Color,
    pub visibility: Visibility,
    pub name: &'static str,
    pub index: VoxelIndex
}

impl Voxel 
{
    pub fn from_index(VoxelIndex(index): VoxelIndex) -> &'static Voxel
    {
        &Self::GAME_VOXELS[index as usize]
    }

    /// Voxel size in meters
    pub const VOXEL_SIZE: f32 = 1.0 / 16.0;
    pub const GAME_VOXELS: &[Voxel] =
    &[
        Voxel
        {
            color: Color::WHITE,
            visibility: Visibility::Empty,
            name: "Empty",
            index: VoxelIndex(0)
        },
        Voxel
        {
            color: Color::BLUE,
            visibility: Visibility::Opaque,
            name: "Water",
            index: VoxelIndex(1)
        },
        Voxel 
        {
            color: Color::new(0.76, 0.698, 0.502, 1.0),
            visibility: Visibility::Opaque,
            name: "Sand",
            index: VoxelIndex(2)
        },
        Voxel
        {
            color: Color::GREEN,
            visibility: Visibility::Opaque,
            name: "Grass",
            index: VoxelIndex(3)
        }
];
}

pub trait VoxelStorage : Sized
{
    fn new(depth: usize) -> Self;
    fn depth(&self) -> usize;
    fn get(&self, index: Vec3<usize>) -> VoxelIndex;
    fn insert(&mut self, index: Vec3<usize>, value: VoxelIndex);
    fn simplify(&mut self);
    fn is_empty(&self) -> bool;

    fn new_from_grid<TArg, TFunc>(depth: usize, grid: &Array3D<TArg>, mut sampler: TFunc) -> Self
        where TFunc : FnMut(&TArg) -> VoxelIndex
    {
        let mut storage = Self::new(depth);
        let length = storage.length();
        assert!(grid.width() == length && grid.height() == length && grid.depth() == length, "Array was not of the proper size.");

        for x in 0..length
        {
            for y in 0..length
            {
                for z in 0..length
                {
                    let voxel = sampler(&grid[Vec3::new(x, y, z)]);
                    storage.insert([x, y, z].into(), voxel);
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

pub trait VoxelStorageExt
{
    fn length(&self) -> usize;
    fn voxel_count(&self) -> usize;
    fn insert_and_simplify(&mut self, index: Vec3<usize>, value: VoxelIndex);
}

impl<TStorage> VoxelStorageExt for TStorage 
    where TStorage : VoxelStorage
{
    fn length(&self) -> usize 
    {
        (2 as usize).pow(self.depth() as u32)
    }

    fn voxel_count(&self) -> usize 
    {
        self.length().pow(3)
    }

    fn insert_and_simplify(&mut self, index: Vec3<usize>, value: VoxelIndex) 
    {
        self.insert(index, value);
        self.simplify();
    }
}

fn get_voxel_faces<TStorage>(data: &TStorage) -> VoxelMesh
    where TStorage : VoxelStorage
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

fn has_face<TStorage>(data: &TStorage, index: Vec3<usize>, face_dir: FaceDir) -> bool
    where TStorage : VoxelStorage
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
                let voxel_index = data.get([index.x, index.y, index.z + 1].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
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
                let voxel_index = data.get([index.x, index.y, index.z - 1].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
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
                let voxel_index = data.get([index.x - 1, index.y, index.z].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
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
                let voxel_index = data.get([index.x + 1, index.y, index.z].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
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
                let voxel_index = data.get([index.x, index.y + 1, index.z].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
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
                let voxel_index = data.get([index.x, index.y - 1, index.z].into());
                Voxel::from_index(voxel_index).visibility != Visibility::Opaque
            }
        },
    }
}

fn add_faces<TStorage>(data: &TStorage, index: Vec3<usize>, mesh: &mut VoxelMesh)
    where TStorage : VoxelStorage
{
    let size = data.length();

    if index.x >= size || index.y >= size || index.z >= size
    {
        panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z);
    }

    let voxel = data.get([index.x, index.y, index.z].into());
    if Voxel::from_index(voxel).visibility == Visibility::Empty { return; }

    let pos = index.cast().unwrap();

    if has_face(data, index, FaceDir::South)
    {
        mesh.add_face(pos, FaceDir::South, voxel.into());
    }

    if has_face(data, index, FaceDir::North)
    {
        mesh.add_face(pos, FaceDir::North, voxel.into());
    }

    if has_face(data, index, FaceDir::East)
    {
        mesh.add_face(pos, FaceDir::East, voxel.into());
    }

    if has_face(data, index, FaceDir::West)
    {
        mesh.add_face(pos, FaceDir::West, voxel.into());
    }

    if has_face(data, index, FaceDir::Up)
    {
        mesh.add_face(pos, FaceDir::Up, voxel.into());
    }

    if has_face(data, index, FaceDir::Down)
    {
        mesh.add_face(pos, FaceDir::Down, voxel.into());
    }
}