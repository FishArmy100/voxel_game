use glam::IVec3;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TerrainArgs
{
    pub chunk_size: u32,
    pub seed: u32, // TODO: support 64 bit seeds
}

impl TerrainArgs
{
    pub fn new(chunk_size: u32, seed: u32) -> Self 
    {
        Self
        {
            chunk_size,
            seed,
        }
    }
}