use std::fmt;

#[derive(Clone, Copy)]
pub struct Point (pub usize, pub usize);

impl Point {
    pub fn origin() -> Self {
        Self(0, 0)
    }

    pub fn new(i: usize, j: usize) -> Self {
        Self(i, j)
    }

    pub fn tuple(&self) -> (usize, usize) {
        (self.0, self.1)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "({}, {})", self.0, self.1)
    }
}
