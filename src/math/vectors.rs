use std::{ops::{Add, Sub, Div, Mul}, fmt::Display};


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vec2<T>
{
    pub x: T,
    pub y: T
}

impl<T> Vec2<T>
{
    pub fn new(x: T, y: T) -> Self
    {
        Vec2 { x, y }
    }

    pub fn to_array(self) -> [T; 2]
    {
        [self.x, self.y]
    }
}

impl<T> Vec2<T> where T : Clone
{
    pub fn from_array(arr: &[T; 2]) -> Self
    {
        Self::new(arr[0].clone(), arr[1].clone())
    }
}

impl<T> Add for Vec2<T> where T : Add
{
    type Output = Vec2<T::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T> Sub for Vec2<T> where T : Sub
{
    type Output = Vec2<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T> Mul<T> for Vec2<T> where T : Mul + Copy
{
    type Output = Vec2<T::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl<T> Div<T> for Vec2<T> where T : Div + Copy
{
    type Output = Vec2<T::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl<T> Display for Vec2<T> where T : Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.x, self.y))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vec3<T>
{
    pub x: T,
    pub y: T,
    pub z: T
}

impl<T> Vec3<T>
{
    pub fn new(x: T, y: T, z: T) -> Self
    {
        Vec3 { x, y, z }
    }

    pub fn to_array(self) -> [T; 3]
    {
        [self.x, self.y, self.z]
    }
}

impl<T> Vec3<T> where T : Clone
{
    pub fn from_array(arr: &[T; 3]) -> Self
    {
        Self::new(arr[0].clone(), arr[1].clone(), arr[2].clone())
    }
}

impl<T> Add for Vec3<T> where T : Add
{
    type Output = Vec3<T::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T> Sub for Vec3<T> where T : Sub
{
    type Output = Vec3<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T> Mul<T> for Vec3<T> where T : Mul + Copy
{
    type Output = Vec3<T::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T> Div<T> for Vec3<T> where T : Div + Copy
{
    type Output = Vec3<T::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T> Display for Vec3<T> where T : Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.x, self.y, self.z))
    }
}

