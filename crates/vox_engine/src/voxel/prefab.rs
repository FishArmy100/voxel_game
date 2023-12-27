use vox_core::VoxelModel;

pub struct VoxelPrefab
{
    pub name: String,
    pub model: VoxelModel,
}

impl VoxelPrefab
{
    pub fn new<T>(name: T, model: VoxelModel) -> Self 
        where T : Into<String>
    {
        Self
        {
            name: name.into(),
            model
        }
    }
}