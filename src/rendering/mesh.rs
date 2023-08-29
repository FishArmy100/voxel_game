use crate::math::*;
use crate::colors::Color;
use crate::rendering::VertexData;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex 
{
    position: Vec3<f32>,
    color: Color
}

impl Vertex
{
    pub fn new(position: Vec3<f32>, color: Color) -> Self
    {
        Self { position, color }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl VertexData for Vertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]).iter());
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle(u16, u16, u16);

unsafe impl bytemuck::Pod for Triangle {}
unsafe impl bytemuck::Zeroable for Triangle {}

pub struct Mesh 
{
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>
}

impl Mesh 
{
    pub fn new(vertices: Vec<Vertex>, triangles: Vec<Triangle>) -> Self
    {
        Self { vertices, triangles }
    }

    pub fn get_triangle_indexes(&self) -> &[u16]
    {
        bytemuck::cast_slice(&self.triangles)
    }

    pub fn cube(color: Color) -> Self
    {
        let vertices = vec![
            Vertex::new(Vec3::new(0., 0., 0.), color),
            Vertex::new(Vec3::new(1., 0., 0.), color),
            Vertex::new(Vec3::new(0., 1., 0.), color),
            Vertex::new(Vec3::new(1., 1., 0.), color),
            Vertex::new(Vec3::new(0., 0., 1.), color),
            Vertex::new(Vec3::new(1., 0., 1.), color),
            Vertex::new(Vec3::new(0., 1., 1.), color),
            Vertex::new(Vec3::new(1., 1., 1.), color),
        ];

        let triangles = vec![
            Triangle(2, 6, 7),
            Triangle(2, 3, 7),

            //Bottom
            Triangle(0, 4, 5),
            Triangle(0, 1, 5),

            //Left
            Triangle(0, 2, 6),
            Triangle(0, 4, 6),

            //Right
            Triangle(1, 3, 7),
            Triangle(1, 5, 7),

            //Front
            Triangle(0, 2, 3),
            Triangle(0, 1, 3),

            //Back
            Triangle(4, 6, 7),
            Triangle(4, 5, 7)
        ];

        Mesh::new(vertices, triangles)
    }
}