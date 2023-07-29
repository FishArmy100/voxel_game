use crate::{utils::Array3D, math::Vec3};

// depth is equal to the number of tree nodes needed to access a point
fn length_from_depth(depth: u32) -> u32
{
    assert!(depth != 0, "Depth cannot be 0");
    (2 as u32).pow(depth - 1)
}

fn offset_3d_of_branch_index(index: u8) -> Vec3<usize>
{
    match index 
    {
        0 => Vec3::new(0, 0, 0),
        1 => Vec3::new(1, 0, 0),
        2 => Vec3::new(0, 1, 0),
        3 => Vec3::new(1, 1, 0),
        4 => Vec3::new(0, 0, 1),
        5 => Vec3::new(1, 0, 1),
        6 => Vec3::new(0, 1, 1),
        7 => Vec3::new(1, 1, 1),
        _ => panic!("Invalid branch index {}", index)
    }
}

pub struct VoxelArray<T> where T : Clone
{
    depth: u32,
    length: u32,
    data: Array3D<T>
}

impl<T> VoxelArray<T> where T : Clone
{
    pub fn new_from_array(depth: u32, data: Array3D<T>) -> Self
    {
        let length = length_from_depth(depth);
        assert!(data.width() == length as usize && data.height() == length as usize && data.depth() == length as usize, "All array dimensions must be 2^{}", depth);

        Self { depth, length, data }
    }

    pub fn new<F>(depth: u32, gen: &F) -> Self 
        where F : Fn(usize, usize, usize) -> T 
    {
        let length = length_from_depth(depth);
        let data = Array3D::new(length as usize, length as usize, length as usize, gen);
        Self { depth, length, data }
    }
}

#[derive(Debug, Clone)]
pub enum VoxelTree<T>
{
    Empty,
    Leaf(T),
    Branches(Box<[VoxelTree<T>; 8]>)
}

impl<T> VoxelTree<T>
{
    // If is a leaf, returns Some(value), else returns None
    pub fn try_get_leaf(&self) -> Option<&T>
    {
        match self
        {
            VoxelTree::Empty => None,
            VoxelTree::Leaf(l) => Some(l),
            VoxelTree::Branches(_) => None,
        }
    }

    
}

impl<T> VoxelTree<T> where T : Clone 
{
    // Generates a full empty tree based on the given depth
    pub fn gen_from_depth(depth: u32) -> Self
    {
        assert!(depth != 0, "Depth cannot be 0");
        if depth > 1
        {
            let current = Self::gen_from_depth(depth - 1);
            Self::Branches(Box::new(
                [
                    current.clone(), 
                    current.clone(), 
                    current.clone(), 
                    current.clone(), 
                    current.clone(), 
                    current.clone(), 
                    current.clone(), 
                    current.clone(),
                ]))
        }
        else
        {
            Self::Empty    
        }
    }
}

impl<T> VoxelTree<T> where T : Clone + PartialEq
{
    pub fn merge(&mut self)
    {
        match self
        {
            VoxelTree::Empty => {},
            VoxelTree::Leaf(_) => todo!(),
            VoxelTree::Branches(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Octree<T> where T : Clone
{
    depth: u32,
    tree: VoxelTree<T>
}

impl<T> Octree<T> where T : Clone
{
    pub fn length(&self) -> u32 { length_from_depth(self.depth) }
    pub fn root(&self) -> &VoxelTree<T> { &self.tree }

    pub fn new_empty(depth: u32) -> Self
    {
        Self { depth, tree: VoxelTree::Empty }
    }

    // pub fn from_voxel_array(array: VoxelArray<T>) -> Self
    // {
    //     let mut tree = VoxelTree::<T>::gen_from_depth(array.depth);

    // }

    fn location_from_tree_indexes(&self, indexes: &[u8]) -> Vec3<usize>
    {
        let mut offset = Vec3::new(0, 0, 0);
        for i in 0..indexes.len()
        {
            let offset_scale = length_from_depth(self.depth - i as u32) as usize;
            offset += offset_3d_of_branch_index(indexes[i]) * offset_scale;
        }

        offset
    }
}