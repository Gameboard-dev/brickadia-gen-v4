
#[derive(Clone)]
pub struct Cell {
    pub inner_wall: bool,
    pub right_wall: bool,
    pub visited: bool,
    pub outer_wall: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            inner_wall: true,
            right_wall: true,
            visited: false,
            outer_wall: true,
        }
    }
}