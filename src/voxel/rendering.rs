
pub enum FaceDir
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl FaceDir
{
    pub fn to_index(&self) -> u32
    {
        match self 
        {
            FaceDir::Up =>      0,
            FaceDir::Down =>    1,
            FaceDir::North =>   2,
            FaceDir::South =>   3,
            FaceDir::East =>    4,
            FaceDir::West =>    5,
        }
    }
}