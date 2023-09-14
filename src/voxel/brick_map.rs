use crate::{utils::Array3D, math::Vec3};

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
    depth: u32,
    data: SubGridData<T>
}

fn get_grid_with_value<T>(length: u32, old: T, new_value: T, new_index: Vec3<u32>) -> Array3D<T> 
    where T : Clone + PartialEq
{
    let mut grid = Array3D::new_with_value(length as usize, length as usize, length as usize, old);
    let index: Vec3<usize> = new_index.cast().unwrap();
    grid[index] = new_value;
    grid
}

impl<T> SubGrid<T> where T : Clone + PartialEq
{
    pub fn voxel_count(&self) -> u32
    {
        self.length().pow(3)
    }

    pub fn length(&self) -> u32
    {
        (2 as u32).pow(self.depth)
    }

    pub fn depth(&self) -> u32 
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

    pub fn get(&self, index: Vec3<u32>) -> Option<T>
    {
        match &self.data
        {
            SubGridData::Empty => None,
            SubGridData::Value(value) => Some(value.clone()),
            SubGridData::Grid(grid) => grid[index.cast().unwrap()].clone(),
        }
    }

    pub fn insert(&mut self, index: Vec3<u32>, inserted: Option<T>)
    {
        let length = self.length();
        assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the sub grid", index);
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
                grid[index.cast().unwrap()] = inserted;
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
    depth: u32,
    sub_grid_depth: u32,
    data: BrickMapData<T>
}

impl<T> BrickMap<T> where T : Clone + PartialEq
{
    pub fn voxel_count(&self) -> u32
    {
        self.length().pow(3)
    }

    pub fn length(&self) -> u32
    {
        (2 as u32).pow(self.depth)
    }

    pub fn depth(&self) -> u32 
    {
        self.depth
    }

    pub fn sub_grid_length(&self) -> u32 
    {
        (2 as u32).pow(self.sub_grid_depth)
    }

    pub fn sub_grid_depth(&self) -> u32 
    {
        self.sub_grid_depth
    }

    pub fn get_sub_grid_index(&self, voxel_index: Vec3<u32>) -> Vec3<u32>
    {
        voxel_index / self.sub_grid_length()
    }

    pub fn get_remainder_index(&self, voxel_index: Vec3<u32>) -> Vec3<u32>
    {
        voxel_index % self.sub_grid_length()
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

    pub fn get(&self, index: Vec3<u32>) -> Option<T>
    {
        match &self.data
        {
            BrickMapData::Empty => None,
            BrickMapData::Value(value) => Some(value.clone()),
            BrickMapData::Grid(grid) => 
            {
                let sub_grid_index = self.get_sub_grid_index(index);
                let remainder_index = index % self.sub_grid_length();
                grid.get(sub_grid_index.cast().unwrap()).get(remainder_index)
            },
        }
    }

    pub fn insert(&mut self, index: Vec3<u32>, inserted: Option<T>)
    {
        let length = self.length();
        let sub_grid_index = self.get_sub_grid_index(index);
        let remainder_index = self.get_remainder_index(index);
        assert!(index.x < length && index.y < length && index.z < length, "Index {:?} is out of bounds of the sub grid", index);
        match &mut self.data
        {
            BrickMapData::Empty => 
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
            BrickMapData::Value(grid_value) => 
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
            BrickMapData::Grid(sub_grid) => 
            {
                sub_grid[sub_grid_index.cast().unwrap()].insert(remainder_index, inserted)
            },
        }
    }
}