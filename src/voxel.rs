pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;
use crate::colors::Color;
use crate::math::Vec3;
use crate::rendering::VertexData;
use crate::rendering::voxel_render_stage::{VoxelFace, VoxelRenderData, VoxelFaceOrientation, VoxelVertex};
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

    fn get_mesh(&self) -> VoxelMesh
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

pub struct VoxelMesh
{
    vertices: Vec<VoxelVertex>,
    triangles: Vec<u32>,
    faces: Vec<VoxelFace>
}

impl VoxelMesh
{
    pub fn vertices(&self) -> &[VoxelVertex] { &self.vertices }
    pub fn triangles(&self) -> &[u32] { &self.triangles }
    pub fn faces(&self) -> &[VoxelFace] { &self.faces }

    pub fn new() -> Self 
    {
        Self 
        {
            vertices: vec![],
            triangles: vec![],
            faces: vec![]
        }
    }

    pub fn add_face(&mut self, pos: Vec3<u32>, orientation: VoxelFaceOrientation, voxel_id: u16)
    {
        let vertex = VoxelVertex {
            face_id: self.faces.len() as u32
        };

        self.vertices.extend([vertex; 4]);
        self.triangles.extend(VOXEL_FACE_TRIANGLES.map(|i| i + 1));

        let face = VoxelFace::new(pos, orientation, voxel_id as u32);
        self.faces.push(face);
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

fn has_face<TStorage, TVoxel>(data: &TStorage, index: Vec3<usize>, face_id: VoxelFaceOrientation) -> bool
    where TStorage : VoxelStorage<TVoxel>, TVoxel : IVoxel
{
    let size = data.length();
    match face_id
    {
        VoxelFaceOrientation::South => 
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
        VoxelFaceOrientation::North => 
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
        VoxelFaceOrientation::West => 
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
        VoxelFaceOrientation::East => 
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
        VoxelFaceOrientation::Up => 
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
        VoxelFaceOrientation::Down => 
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
    let pos = Vec3::new(index.x as u32, index.y as u32, index.z as u32);

    if has_face(data, index, VoxelFaceOrientation::South)
    {
        mesh.add_face(pos, VoxelFaceOrientation::South, voxel.id());
    }

    if has_face(data, index, VoxelFaceOrientation::North)
    {
        mesh.add_face(pos, VoxelFaceOrientation::North, voxel.id());
    }

    if has_face(data, index, VoxelFaceOrientation::East)
    {
        mesh.add_face(pos, VoxelFaceOrientation::East, voxel.id());
    }

    if has_face(data, index, VoxelFaceOrientation::West)
    {
        mesh.add_face(pos, VoxelFaceOrientation::West, voxel.id());
    }

    if has_face(data, index, VoxelFaceOrientation::Up)
    {
        mesh.add_face(pos, VoxelFaceOrientation::Up, voxel.id());
    }

    if has_face(data, index, VoxelFaceOrientation::Down)
    {
        mesh.add_face(pos, VoxelFaceOrientation::Down, voxel.id());
    }
}