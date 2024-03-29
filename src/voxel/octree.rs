use cgmath::{Array, Zero};

use crate::{math::Vec3, utils::{self, Array3D}};

use super::{VoxelStorage, IVoxel};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Octant
{
    LeftRearBase    = 0,
    RightRearBase   = 1,
    LeftRearTop     = 2,
    RightRearTop    = 3,
    LeftFrontBase   = 4,
    RightFrontBase  = 5,
    LeftFrontTop    = 6,
    RightFrontTop   = 7,
}

impl TryFrom<usize> for Octant
{
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::LeftRearBase),
            1 => Ok(Self::RightRearBase),
            2 => Ok(Self::LeftRearTop),
            3 => Ok(Self::RightRearTop),
            4 => Ok(Self::LeftFrontBase),
            5 => Ok(Self::RightFrontBase),
            6 => Ok(Self::LeftFrontTop),
            7 => Ok(Self::RightFrontTop),
            _ => Err(()),
        }
    }
}

pub struct Octree<T> where T : Copy + Clone + Eq
{
    depth: usize,
    root: Node<T>
}

impl<T> VoxelStorage<T> for Octree<T> where T : IVoxel + Copy + PartialEq
{
    fn new(depth: usize) -> Self 
    {
        let bounds = NodeBounds::new_from_max(depth);
        let root = Node::new(bounds);

        Self { depth, root }
    }

    fn depth(&self) -> usize 
    {
        self.depth
    }

    fn get(&self, index: Vec3<usize>) -> Option<T> 
    {
        self.root.get(index)
    }

    fn insert(&mut self, index: Vec3<usize>, value: Option<T>) 
    {
        self.root.insert(index, value);
    }

    fn simplify(&mut self) 
    {
        self.root.simplify();
    }

    fn is_empty(&self) -> bool 
    {
        match self.root.data 
        {
            NodeType::Empty => true,
            NodeType::Leaf(_) => false,
            NodeType::Branches(_) => false,
        }
    }

    fn new_from_grid<TArg, TFunc>(depth: usize, grid: &Array3D<TArg>, mut sampler: TFunc) -> Self
        where TFunc : FnMut(&TArg) -> Option<T> 
    {
        let bounds = NodeBounds::new_from_max(depth);
        let mut node = Node::new(bounds);
        fill_node_from_grid(&mut node, &grid, &mut sampler);
        node.simplify();

        Self 
        { 
            depth, 
            root: node 
        }
    }

    // fn get_faces(&self, position: Vec3<isize>) -> Vec<VoxelFaceData> 
    // {
    //     let mut faces = vec![];
    //     stupid_get_faces(&self.root, &mut faces, position);
    //     faces
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeBounds
{
    max_depth: usize,
    current_depth: usize,
    position_relative: Vec3<usize>
}

impl NodeBounds
{
    fn new_from_max(max_depth: usize) -> Self
    {
        Self { max_depth, current_depth: 0, position_relative: Vec3::from_value(0) }
    }

    fn contains_point(&self, point: Vec3<usize>) -> bool
    {
        let (position, sub_voxel_size) = self.get_bounds_location();

        point.x >= position.x && point.x < position.x + sub_voxel_size &&
        point.y >= position.y && point.y < position.y + sub_voxel_size &&
        point.z >= position.z && point.z < position.z + sub_voxel_size
    }

    fn get_child_bounds(&self) -> [Self; 8]
    {
        assert!(!self.is_max_depth(), "Cannot get children from max depth node bounds");
        let child_depth = self.current_depth + 1;

        [
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(0, 0, 0) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(1, 0, 0) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(0, 1, 0) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(1, 1, 0) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(0, 0, 1) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(1, 0, 1) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(0, 1, 1) },
            Self { max_depth: self.max_depth, current_depth: child_depth, position_relative: self.position_relative * 2 + Vec3::new(1, 1, 1) },
        ]
    }

    fn get_bounds_location(&self) -> (Vec3<usize>, usize)
    {
        let sub_voxel_size = (2 as usize).pow(self.max_depth as u32) / (2 as usize).pow(self.current_depth as u32);
        let position = self.position_relative * sub_voxel_size;

        (position, sub_voxel_size)
    }

    fn is_max_depth(&self) -> bool
    {
        self.current_depth == self.max_depth
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeType<T> where T : Copy + Clone + Eq
{
    Empty,
    Leaf(T),
    Branches(Box<[Node<T>; 8]>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node<T> where T : Copy + Clone + Eq
{
    data: NodeType<T>,
    bounds: NodeBounds, 
}

impl<T> Node<T> where T : Copy + Clone + Eq
{
    fn new(bounds: NodeBounds) -> Self
    {
        Self { data: NodeType::Empty, bounds }
    }

    fn insert(&mut self, index: Vec3<usize>, value: Option<T>)
    {
        if self.bounds.is_max_depth()
        {
            assert!(index == self.bounds.position_relative, "thought {:?} was {:?}", index, self.bounds.position_relative);
            match value 
            {
                Some(value) => self.data = NodeType::Leaf(value),
                None => self.data = NodeType::Empty,
            }
        }
        else
        {
            let child_index = self.get_child_index(index);
            match &mut self.data
            {
                NodeType::Empty =>
                {
                    let mut branches = Box::new(self.get_empty_children(None));

                    branches[child_index].insert(index, value);

                    self.data = NodeType::Branches(branches);
                },
                NodeType::Leaf(leaf) => 
                {
                    let leaf = *leaf;
                    let mut branches = Box::new(self.get_empty_children(Some(leaf)));
                    
                    branches[child_index].insert(index, value);

                    self.data = NodeType::Branches(branches);
                },
                NodeType::Branches(branches) => 
                {
                    branches[child_index].insert(index, value);
                }
            }
        }
    }

    fn get(&self, index: Vec3<usize>) -> Option<T>
    {
        assert!(self.bounds.contains_point(index));
        match &self.data 
        {
            NodeType::Empty => None,
            NodeType::Leaf(leaf) => Some(*leaf),
            NodeType::Branches(branches) => 
            {
                branches.iter().find(|b| b.bounds.contains_point(index)).unwrap().get(index)
            },
        }
    }

    fn get_empty_children(&self, value: Option<T>) -> [Node<T>; 8]
    {
        let child_bounds = self.bounds.get_child_bounds();
        let children: [Node<T>; 8] = std::array::from_fn(|i| {
            let data = match value 
            {
                Some(leaf) => NodeType::Leaf(leaf),
                None => NodeType::Empty,
            };
            Self { data, bounds: child_bounds[i] }
        });

        children
    }

    fn get_child_index(&self, position: Vec3<usize>) -> usize
    {
        let sub_octant_len = (2 as u32).pow((self.bounds.max_depth - self.bounds.current_depth - 1) as u32);
        let current_position = position / sub_octant_len as usize;

        let relative = current_position - self.bounds.position_relative * 2;

        assert!(relative.x < 2 && relative.y < 2 && relative.z < 2, "Index error");
        let index = utils::index_3d_to_index_1d(2, 2, 2, relative);

        index
    }

    fn simplify(&mut self) -> bool
    {
        match &mut self.data 
        {
            NodeType::Branches(branches) => 
            {
                let was_simplified = branches.iter_mut().all(|b| b.simplify());
                if was_simplified
                {
                    let first = branches[0].clone(); // should never clone a `Branches` node
                    let are_all_same = branches.iter().all(|b| b.data == first.data);
                    if are_all_same
                    {
                        self.data = first.data;
                        true
                    }
                    else 
                    {
                        false
                    }
                }
                else 
                {
                    false
                }
            },
            _ => true
        }
    }
}

fn fill_node_from_grid<T, A, S>(node: &mut Node<T>, grid: &Array3D<A>, sampler: &mut S) 
    where T : Copy + Clone + Eq,
          S : FnMut(&A) -> Option<T>
{
    if !node.bounds.is_max_depth()
    {
        let children = node.bounds.get_child_bounds().map(|b| {
            let mut child: Node<T> = Node::new(b);
            fill_node_from_grid(&mut child, grid, sampler);
            child
        });

        node.data = NodeType::Branches(Box::new(children));
    }
    else 
    {
        let voxel = sampler(&grid[node.bounds.position_relative]);

        let new_data = match voxel {
            Some(value) => NodeType::Leaf(value),
            None => NodeType::Empty,
        };

        node.data = new_data;
    }
}