use std::fmt;

#[derive(Clone, Copy)]
pub struct Point {
    i: usize,
    j: usize
}

impl Point {
    pub fn origin() -> Self {
        Self {i:0, j:0}
    }

    pub fn new(i: usize, j: usize) -> Self {
        Self {i, j}
    }

    pub fn tuple(&self) -> (usize, usize) {
        (self.i, self.j)
    }

    #[allow(dead_code)]
    pub fn arr(&self) -> [usize; 2] {
        [self.i, self.j]
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "({}, {})", self.i, self.j)
    }
}