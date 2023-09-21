use crate::{utils::Array3D, math::Vec3, rendering::voxel_render_stage::{VoxelFaceData, VoxelFace}};

use super::{VoxelStorage, IVoxel};

#[derive(Debug, Clone, PartialEq)]
enum SubGridData<T> where T : Clone + PartialEq
{
    Empty,
    Value(T),
    Grid(Array3D<Option<T>>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubGrid<T> where T : Clone + PartialEq
{
    depth: usize,
    data: SubGridData<T>
}

fn get_grid_with_value<T>(length: usize, old: T, new_value: T, new_index: Vec3<usize>) -> Array3D<T> 
    where T : Clone + PartialEq
{
    let mut grid = Array3D::new_with_value(length, length, length, old);
    grid[new_index] = new_value;
    grid
}

impl<T> SubGrid<T> where T : Clone + PartialEq
{
    pub fn voxel_count(&self) -> usize
    {
        self.length().pow(3)
    }

    pub fn length(&self) -> usize
    {
        (2 as usize).pow(self.depth as u32)
    }

    pub fn depth(&self) -> usize 
    {
        self.depth
    }

    pub fn simplify(&mut self)
    {
        match &mut self.data 
        {
            SubGridData::Empty => {},
            SubGridData::Value(_) => {},
            SubGridData::Grid(grid) => 
            {
                let first = grid[Vec3::new(0, 0, 0)].clone();
                if grid.as_slice().iter().all(|i| *i == first)
                {
                    match first
                    {
                        Some(value) => self.data = SubGridData::Value(value),
                        None => self.data = SubGridData::Empty,
                    }
                }
            },
        }
    }

    pub fn get(&self, index: Vec3<usize>) -> Option<T>
    {
        let length = self.length();
        debug_assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the sub grid", index);
        match &self.data
        {
            SubGridData::Empty => None,
            SubGridData::Value(value) => Some(value.clone()),
            SubGridData::Grid(grid) => grid[index].clone(),
        }
    }

    pub fn insert(&mut self, index: Vec3<usize>, inserted: Option<T>)
    {
        let length = self.length();
        debug_assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the sub grid", index);
        match &mut self.data
        {
            SubGridData::Empty => 
            {
                match inserted
                {
                    Some(_) => 
                    {
                        let grid = get_grid_with_value(length, None, inserted, index);
                        self.data = SubGridData::Grid(grid);
                    },
                    None => {},
                }
            },
            SubGridData::Value(grid_value) => 
            {
                match inserted
                {
                    Some(inserted) => 
                    {
                        if *grid_value != inserted
                        {
                            let grid = get_grid_with_value(length, Some(grid_value.clone()), Some(inserted), index);
                            self.data = SubGridData::Grid(grid);
                        }
                    },
                    None => 
                    {
                        let grid = get_grid_with_value(length, Some(grid_value.clone()), None, index);
                        self.data = SubGridData::Grid(grid);
                    },
                }
            },
            SubGridData::Grid(grid) => 
            {
                grid[index] = inserted;
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BrickMapData<T> where T : Clone + PartialEq
{
    Empty,
    Value(T),
    Grid(Array3D<SubGrid<T>>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrickMap<T> where T : Clone + PartialEq
{
    depth: usize,
    sub_grid_depth: usize,
    data: BrickMapData<T>
}

impl<T> BrickMap<T> where T : Clone + PartialEq
{
    pub fn voxel_count(&self) -> usize
    {
        self.length().pow(3)
    }

    pub fn length(&self) -> usize
    {
        (2 as usize).pow(self.depth as u32)
    }

    pub fn depth(&self) -> usize 
    {
        self.depth
    }

    pub fn sub_grid_length(&self) -> usize 
    {
        (2 as usize).pow(self.sub_grid_depth as u32)
    }

    pub fn sub_grid_depth(&self) -> usize 
    {
        self.sub_grid_depth
    }

    pub fn get_sub_grid_index(&self, voxel_index: Vec3<usize>) -> Vec3<usize>
    {
        voxel_index / self.sub_grid_length()
    }

    pub fn get_remainder_index(&self, voxel_index: Vec3<usize>) -> Vec3<usize>
    {
        voxel_index % self.sub_grid_length()
    }

    pub fn new(depth: usize, sub_grid_depth: usize, default_value: Option<T>) -> Self
    {
        assert!(depth > sub_grid_depth, "Grid depth must be larger than the sub grid depth");
        
        let data = match default_value {
            Some(val) => BrickMapData::Value(val),
            None => BrickMapData::Empty,
        };
        
        Self 
        { 
            depth, 
            sub_grid_depth, 
            data 
        }
    }

    pub fn simplify(&mut self)
    {
        match &mut self.data 
        {
            BrickMapData::Empty => {},
            BrickMapData::Value(_) => {},
            BrickMapData::Grid(grid) => 
            {
                for sub_grid in grid.as_mut_slice()
                {
                    sub_grid.simplify();
                }

                let first = &grid[Vec3::new(0, 0, 0)];
                match &first.data
                {
                    SubGridData::Empty => 
                    {
                        if grid.as_slice().iter().all(|sg| sg.data == SubGridData::Empty)
                        {
                            self.data = BrickMapData::Empty;
                        }
                    },
                    SubGridData::Value(val) =>
                    {
                        if grid.as_slice().iter().all(|sg| sg.data == SubGridData::Value(val.clone()))
                        {
                            self.data = BrickMapData::Value(val.clone());
                        }
                    },
                    SubGridData::Grid(_) => {},
                }
            },
        }
    }    

    pub fn get(&self, index: Vec3<usize>) -> Option<T>
    {
        let length = self.length();
        debug_assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the brick map", index);
        
        match &self.data
        {
            BrickMapData::Empty => None,
            BrickMapData::Value(value) => Some(value.clone()),
            BrickMapData::Grid(grid) => 
            {
                let sub_grid_index = self.get_sub_grid_index(index);
                let remainder_index = self.get_remainder_index(index);
                grid.get(sub_grid_index).get(remainder_index)
            },
        }
    }

    pub fn insert(&mut self, index: Vec3<usize>, inserted: Option<T>)
    {
        let length = self.length();
        let sub_grid_depth = self.sub_grid_depth();
        let sub_grid_index = self.get_sub_grid_index(index);
        let remainder_index = self.get_remainder_index(index);

        debug_assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the grid", index);
        match &mut self.data
        {
            BrickMapData::Empty => 
            {
                match inserted
                {
                    Some(_) => 
                    {
                        let sub_grid_array = self.get_brick_map(None, inserted, index);
                        self.data = BrickMapData::Grid(sub_grid_array);
                    },
                    None => {},
                }
            },
            BrickMapData::Value(grid_value) => 
            {
                match inserted
                {
                    Some(inserted) => 
                    {
                        if *grid_value != inserted
                        {   
                            let grid_value = Some(grid_value.clone());
                            let sub_grid_array = self.get_brick_map(grid_value, Some(inserted), index);
                            self.data = BrickMapData::Grid(sub_grid_array);
                        }
                    },
                    None => 
                    {
                        let grid_value = Some(grid_value.clone());
                        let sub_grid_array = self.get_brick_map(grid_value, None, index);
                        self.data = BrickMapData::Grid(sub_grid_array);
                    },
                }
            },
            BrickMapData::Grid(sub_grid) => 
            {
                sub_grid[sub_grid_index].insert(remainder_index, inserted)
            },
        }
    }

    
    fn get_brick_map(&self, old_value: Option<T>, new_value: Option<T>, new_index: Vec3<usize>) -> Array3D<SubGrid<T>>
        where T : Clone + PartialEq
    {
        let sub_grid_data = match old_value {
            Some(val) => SubGridData::Value(val),
            None => SubGridData::Empty,
        };

        let sub_grid = SubGrid {
            depth: self.sub_grid_depth,
            data: sub_grid_data 
        };

        let sub_count = (2 as usize).pow((self.depth - self.sub_grid_depth) as u32);
        let mut sub_grid_array = Array3D::new_with_value(sub_count, sub_count, sub_count, sub_grid);

        let sub_grid_index = self.get_sub_grid_index(new_index);
        let remainder_index = self.get_remainder_index(new_index);
        sub_grid_array[sub_grid_index].insert(remainder_index, new_value);

        sub_grid_array
    }
}

pub struct SizedBrickMap<T, const D: usize> where T : IVoxel
{
    map: BrickMap<T>
}

impl<T, const D: usize> VoxelStorage<T> for SizedBrickMap<T, D> where T : IVoxel
{
    fn new(depth: usize) -> Self 
    {
        Self 
        {
            map: BrickMap::new(depth, D, None)
        }
    }

    fn depth(&self) -> usize 
    {
        self.map.depth
    }

    fn get(&self, index: Vec3<usize>) -> Option<T> 
    {
        self.map.get(index)
    }

    fn insert(&mut self, index: Vec3<usize>, value: Option<T>) 
    {
        self.map.insert(index, value);
    }

    fn simplify(&mut self) 
    {
        self.map.simplify();
    }

    fn is_empty(&self) -> bool 
    {
        match &self.map.data
        {
            BrickMapData::Empty => true,
            BrickMapData::Value(_) => false,
            BrickMapData::Grid(_) => false,
        }
    }

    fn new_from_grid<TArg, TFunc>(depth: usize, grid: &Array3D<TArg>, mut sampler: TFunc) -> Self
            where TFunc : FnMut(&TArg) -> Option<T> 
    {
        let length = (2 as usize).pow(depth as u32);
        debug_assert!(grid.width() == length && grid.height() == length && grid.depth() == length, "Grid initalization array is not of the right size");
        let brick_map = gen_brick_map_from_grid(depth, D, grid, sampler);

        Self
        {
            map: brick_map
        }
    }
}

fn gen_brick_map_from_grid<T, A, S>(depth: usize, sub_grid_depth: usize, grid: &Array3D<A>, mut sampler: S) -> BrickMap<T>
    where T : IVoxel,
          S : FnMut(&A) -> Option<T>
{
    let sub_length = (2 as usize).pow(sub_grid_depth as u32);
    let sub_grid_count = (2 as usize).pow((depth - sub_grid_depth) as u32);

    let brick_map_array = Array3D::new(sub_grid_count, sub_grid_count, sub_grid_count, |x, y, z| {
        let current_index = Vec3::new(x, y, z) * sub_length;
        gen_sub_grid(current_index, sub_grid_depth, grid, &mut sampler)
    });

    let mut brick_map = BrickMap
    {
        depth,
        sub_grid_depth,
        data: BrickMapData::Grid(brick_map_array)
    };

    brick_map.simplify();
    brick_map
}

fn gen_sub_grid<T, S, A>(current_index: Vec3<usize>, sub_depth: usize, grid: &Array3D<A>, sampler: &mut S) -> SubGrid<T>
    where T : IVoxel,
          S : FnMut(&A) -> Option<T>
{
    let sub_length = (2 as usize).pow(sub_depth as u32);
    let sub_grid_array = Array3D::new(sub_length, sub_length, sub_length, |x, y, z| {
        let index = Vec3::new(x, y, z) + current_index;
        sampler(&grid[index])
    });

    SubGrid 
    { 
        depth: sub_depth, 
        data: SubGridData::Grid(sub_grid_array) 
    }
}