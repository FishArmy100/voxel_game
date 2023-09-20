pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
use crate::colors::Color;
use crate::math::Vec3;
use crate::rendering::voxel_render_stage::{VoxelFaceData, VoxelRenderData, VoxelFace};

pub trait VoxelStorage<T> : Sized where T : IVoxel
{
    fn new(depth: usize) -> Self;
    fn depth(&self) -> usize;
    fn get(&self, index: Vec3<usize>) -> Option<T>;
    fn insert(&mut self, index: Vec3<usize>, value: Option<T>);
    fn simplify(&mut self);
    fn is_empty(&self) -> bool;

    fn get_faces(&self, position: Vec3<isize>) -> Vec<VoxelFaceData>
    {
        get_voxel_faces(self, position)
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

fn get_voxel_faces<TStorage, TVoxel>(data: &TStorage, position: Vec3<isize>) -> Vec<VoxelFaceData>
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let mut faces = vec![];

    let length = data.length();
    for x in 0..length
    {
        for y in 0..length
        {
            for z in 0..length 
            {
                add_faces(data, Vec3::new(x, y, z), position, &mut faces);
            }
        }
    }

    faces
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

fn add_faces<TStorage, TVoxel>(data: &TStorage, index: Vec3<usize>, chunk_pos: Vec3<isize>, faces: &mut Vec<VoxelFaceData>)
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let size = data.length();

    if index.x >= size || index.y >= size || index.z >= size
    {
        panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", index.x, index.y, index.z);
    }

    let Some(voxel) = data.get([index.x, index.y, index.z].into()) else { return; };

    let pos = chunk_pos.map(|v| v as i32) + Vec3::new(index.x as i32, index.y as i32, index.z as i32);

    if has_face(data, index, VoxelFace::South)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::South.to_index(), 1);
        faces.push(face);
    }

    if has_face(data, index, VoxelFace::North)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::North.to_index(), 1);
        faces.push(face);
    }

    if has_face(data, index, VoxelFace::East)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::East.to_index(), 1);
        faces.push(face);
    }

    if has_face(data, index, VoxelFace::West)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::West.to_index(), 1);
        faces.push(face);
    }

    if has_face(data, index, VoxelFace::Up)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::Up.to_index(), 1);
        faces.push(face);
    }

    if has_face(data, index, VoxelFace::Down)
    {
        let face = VoxelFaceData::new(pos, voxel.id() as u32, VoxelFace::Down.to_index(), 1);
        faces.push(face);
    }
}