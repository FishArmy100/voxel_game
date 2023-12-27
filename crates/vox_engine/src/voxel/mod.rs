pub mod prefab;
pub mod voxel_renderer;

use glam::{Vec4, vec4};
use vox_core::{camera::RTCameraInfo, VoxelModelInstance, VoxelModel};

use crate::{
    prelude::Array3D, 
    utils::Wrappable,
};

unsafe impl Wrappable for RTCameraInfo {}
unsafe impl Wrappable for VoxelModelInstance {}

#[derive(Debug, Clone, Copy)]
pub struct Voxel 
{
    pub name: &'static str,
    pub id: u32,
    pub color: Vec4
}

pub const VOXELS: &[Voxel] = &[
    Voxel {
        name: "air",
        id: 0,
        color: vec4(1.0, 1.0, 1.0, 1.0)
    },
    Voxel {
        name: "dirt",
        id: 1,
        color: vec4(69.0 / 255.0, 45.0 / 255.0, 45.0 / 255.0, 1.0)
    },
    Voxel {
        name: "grass",
        id: 2,
        color: vec4(93.0 / 255.0, 146.0 / 255.0, 77.0 / 255.0, 1.0)
    },
    Voxel {
        name: "granite",
        id: 3,
        color: vec4(136.0 / 255.0, 140.0 / 255.0, 141.0 / 255.0, 1.0)
    },
    Voxel {
        name: "sandstone",
        id: 4,
        color: vec4(184.0 / 255.0, 176.0 / 255.0, 155.0 / 255.0, 1.0)
    },
    Voxel {
        name: "tree bark",
        id: 5,
        color: vec4(105.0 / 255.0, 75.0 / 255.0, 53.0 / 255.0, 1.0)
    },
    Voxel {
        name: "tree leaves",
        id: 6,
        color: vec4(95.0 / 255.0, 146.0 / 255.0, 106.0 / 255.0, 1.0)
    },
    Voxel {
        name: "water",
        id: 7,
        color: vec4(28.0 / 255.0, 163.0 / 255.0, 236.0 / 255.0, 1.0)
    },
    Voxel {
        name: "error",
        id: 8,
        color: vec4(1.0, 0.0, 1.0, 1.0)
    }
];

pub const AIR:          &Voxel = &VOXELS[0];
pub const DIRT:         &Voxel = &VOXELS[1];
pub const GRASS:        &Voxel = &VOXELS[2];
pub const GRANITE:      &Voxel = &VOXELS[3];
pub const SANDSTONE:    &Voxel = &VOXELS[4];
pub const TREE_BARK:    &Voxel = &VOXELS[5];
pub const TREE_LEAVES:  &Voxel = &VOXELS[6];
pub const WATER:        &Voxel = &VOXELS[7];
pub const ERROR:        &Voxel = &VOXELS[8];

pub fn voxel_colors() -> Vec<Vec4>
{
    VOXELS.iter().map(|v| v.color).collect()
}

pub fn build_vox_model<F>(bytes: &[u8], start_index: u32, mut index_converter: F) -> Result<(VoxelModel, Array3D<u32>), &'static str>
    where F : FnMut(u8) -> u32
{
    let data = match dot_vox::load_bytes(bytes)
    {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let model = match data.models.first()
    {
        Some(m) => m,
        None => return Err(".vox data does not have a model"),
    };

    let mut unique = vec![];
    
    let voxel_model = VoxelModel::new(model.size.x, model.size.z, model.size.y, start_index);
    let mut voxel_array = Array3D::new_with_value(voxel_model.dim_x() as usize, voxel_model.dim_y() as usize, voxel_model.dim_z() as usize, 0);
    for v in &model.voxels
    {
        let color_index = v.i + 1;
        if !unique.contains(&(color_index as u32))
        {
            unique.push(color_index as u32)
        }

        let index = (v.x as usize, v.z as usize, v.y as usize);
        voxel_array[index] = index_converter(color_index);
    }

    println!("unique voxel ids: {:?}", unique);

    Ok((voxel_model, voxel_array))
}

pub fn build_voxel_models<F>(vox_files: &[&[u8]], mut index_converter: F) -> Result<(Vec<VoxelModel>, Vec<u32>), &'static str>
    where F : FnMut(u8) -> u32
{
    let mut models = vec![];
    let mut voxels = vec![];

    for f in vox_files
    {
        let (model, vs) = build_vox_model(f, voxels.len() as u32, &mut index_converter)?;
        models.push(model);
        voxels.extend(vs.as_slice());
    }

    Ok((models, voxels))
}