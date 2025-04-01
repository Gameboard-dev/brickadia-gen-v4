
use super::cast::cast;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    /// Constructs Points from types converted into `i32` iterators.
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = i32>,
    {
        let mut iter = iter.into_iter();
        let x = iter.next().expect("ERROR: Points != 2");
        let y = iter.next().expect("ERROR: Points != 2");
        Self { x, y }
    }
    pub fn from_f32(tuple: (f32, f32)) -> Self {
        Self::new(tuple.0.round() as i32, tuple.1.round() as i32)
    }
    pub fn tuple<T>(&self) -> (T, T)
    where
        T: num_traits::NumCast,
    {
        (cast(self.x), cast(self.y))
    }
}

