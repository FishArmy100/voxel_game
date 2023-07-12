use std::sync::Arc;

use cgmath::{Array, EuclideanSpace};

use crate::colors::Color;
use crate::math::{Vec3, Point3D};
use crate::rendering::{Mesh, Vertex, Triangle, Model, Renderer};

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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
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

enum FaceType
{
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom
}

struct VoxelFace
{
    pub triangles: [Triangle; 2],
    pub vertices: [Vertex; 4]
}

impl VoxelFace
{
    fn back(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 0.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 0.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 0.0, 0.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([0 + current_index, 1 + current_index, 2 + current_index]), Triangle::new([1 + current_index, 3 + current_index, 2 + current_index])];
        Self { triangles, vertices }
    }

    fn front(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 1.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 0.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 0.0, 1.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([2 + current_index, 1 + current_index, 0 + current_index]), Triangle::new([2 + current_index, 3 + current_index, 1 + current_index])];
        Self { triangles, vertices }
    }

    fn right(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 0.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 0.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 0.0, 1.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([0 + current_index, 1 + current_index, 2 + current_index]), Triangle::new([1 + current_index, 3 + current_index, 2 + current_index])];
        Self { triangles, vertices }
    }

    fn left(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 0.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 0.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 0.0, 1.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([2 + current_index, 1 + current_index, 0 + current_index]), Triangle::new([2 + current_index, 3 + current_index, 1 + current_index])];
        Self { triangles, vertices }
    }

    fn top(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 0.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 1.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([2 + current_index, 1 + current_index, 0 + current_index]), Triangle::new([2 + current_index, 3 + current_index, 1 + current_index])];
        Self { triangles, vertices }
    }

    fn bottom(current_index: u16, current_pos: Point3D<f32>, voxel_size: f32, color: Color) -> Self
    {
        let vertices = 
        [
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 0.0) * voxel_size, color), 
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 0.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(0.0, 1.0, 1.0) * voxel_size, color),
            Vertex::new(current_pos + Vec3::new(1.0, 1.0, 1.0) * voxel_size, color)
        ];

        let triangles = [Triangle::new([0 + current_index, 1 + current_index, 2 + current_index]), Triangle::new([1 + current_index, 3 + current_index, 2 + current_index])];
        Self { triangles, vertices }
    }
}

type VoxelArray<const S: usize> = [[[Voxel; S]; S]; S];

pub struct Chunk<const S: usize>
{
    data: VoxelArray<S>,
    position: Point3D<f32>,
    voxels: Arc<Vec<VoxelData>>,
    voxel_size: f32,
    model: Model,
}

impl<const S: usize> Chunk<S>
{
    pub fn new<F>(generator: &F, position: Point3D<f32>, voxels: Arc<Vec<VoxelData>>, voxel_size: f32) -> Self 
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

        let mesh = Self::get_mesh(&data, &voxels, voxel_size);
        let model = Model::new(mesh, position.to_vec());

        Self 
        {
            data,
            position,
            voxels,
            voxel_size,
            model
        }
    }

    pub fn model(&self) -> &Model { &self.model }

    fn has_face(data: &VoxelArray<S>, voxels: &Vec<VoxelData>, x: usize, y: usize, z: usize, face_type: FaceType) -> bool
    {
        match face_type
        {
            FaceType::Front => 
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
            FaceType::Back => 
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
            FaceType::Left => 
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
            FaceType::Right => 
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
            FaceType::Top => 
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
            FaceType::Bottom => 
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
        }
    }

    fn add_faces(data: &VoxelArray<S>, voxels: &Vec<VoxelData>, voxel_size: f32, x: usize, y: usize, z: usize, vertices: &mut Vec<Vertex>, triangles: &mut Vec<Triangle>)
    {
        if x >= S || y >= S || z >= S
        {
            panic!("Index (x: {}, y: {}, z: {}) is not inside the chunk", x, y, z);
        }

        let id = data[x][y][z].id;
        let color = if voxels[id as usize].is_solid
        {
            voxels[id as usize].color
        }
        else 
        {
            return;
        };

        let pos = Point3D::from_value(0.0) + (Vec3::new(x as f32, y as f32, z as f32) * voxel_size);

        if Self::has_face(data, voxels, x, y, z, FaceType::Front)
        {
            let face = VoxelFace::front(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }

        if Self::has_face(data, voxels, x, y, z, FaceType::Back)
        {
            let face = VoxelFace::back(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }

        if Self::has_face(data, voxels, x, y, z, FaceType::Left)
        {
            let face = VoxelFace::left(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }

        if Self::has_face(data, voxels, x, y, z, FaceType::Right)
        {
            let face = VoxelFace::right(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }

        if Self::has_face(data, voxels, x, y, z, FaceType::Top)
        {
            let face = VoxelFace::top(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }

        if Self::has_face(data, voxels, x, y, z, FaceType::Bottom)
        {
            let face = VoxelFace::bottom(vertices.len() as u16, pos, voxel_size, color);
            vertices.extend(face.vertices.iter());
            triangles.extend(face.triangles.iter());
        }
    }

    fn get_mesh(data: &VoxelArray<S>, voxels: &Vec<VoxelData>, voxel_size: f32) -> Mesh
    {
        let mut vertices = vec![];
        let mut triangles = vec![];

        for x in 0..S
        {
            for y in 0..S 
            {
                for z in 0..S
                {
                    Self::add_faces(data, voxels, voxel_size, x, y, z, &mut vertices, &mut triangles);
                }
            }
        }

        Mesh::new(vertices, triangles)
    }
}

pub struct VoxelTerrain<const S: usize = 16>
{
    chunks: Vec<Chunk<S>>,
    position: Point3D<f32>
}

impl<const S: usize> VoxelTerrain<S>
{
    pub const fn chunk_size() -> usize {S}

    pub fn new<F>(position: Point3D<f32>, size_in_chunks: Vec3<usize>, voxel_size: f32, voxel_types: Arc<Vec<VoxelData>>, generator: &F) -> Self
        where F : Fn(Vec3<usize>) -> Voxel
    {
        let mut chunks = vec![];

        for chunk_x in 0..size_in_chunks.x
        {
            for chunk_y in 0..size_in_chunks.y
            {
                for chunk_z in 0..size_in_chunks.z
                {
                    let generator = |x, y, z| generator(Vec3::new(x + chunk_x * S, y + chunk_y * S, z + chunk_z * S));
                    let chunk_pos = Point3D::new(chunk_x as f32, chunk_y as f32, chunk_z as f32) * (S as f32 * voxel_size) + position.to_vec();
                    
                    let chunk = Chunk::<S>::new(&generator, chunk_pos, voxel_types.clone(), voxel_size);
                    chunks.push(chunk);
                }
            }
        }

        Self 
        { 
            chunks, 
            position 
        }
    }

    pub fn render<'s, 'd, 'q, 'c, 'ms>(&'ms self, renderer: &mut Renderer<'s, 'd, 'q, 'c, 'ms>)
    {
        for chunk in &self.chunks
        {
            if chunk.model.mesh.vertices.len() > 0
            {
                renderer.add_model(&chunk.model);
            }
        }
    }
}