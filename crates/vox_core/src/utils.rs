use glam::{UVec3, uvec3};


pub fn flatten_index(index: UVec3, dim: UVec3) -> u32 
{
    (index.z * dim.x * dim.y) + (index.y * dim.x) + index.x
}

pub fn extend_index(mut index: u32, dim: UVec3) -> UVec3
{
    let z = index / (dim.x * dim.y);
    index -= z * dim.x * dim.y;
    let y = index / dim.x;
    let x = index % dim.x;

    uvec3(x, y, z)
}