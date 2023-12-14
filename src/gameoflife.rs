use std::fmt;

use ndarray::{s,Array,Array2};
use rand::distributions::{Distribution,Bernoulli};

pub struct GameOfLife {
    grid: Array2<bool>,
    nstep: u32
}

impl GameOfLife {
    ///////////////
    // Constructors
    ///////////////
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            grid: Array2::default([height, width]),
            nstep: 0
        }
    }

    pub fn random(height: usize, width: usize, fill_ratio: f64) -> Self{
        // ncell = number of cells in grid
        let  ncell = height * width;
        // make bernoulli iterator, capped at [ncell] values
        let rng = rand::thread_rng();
        let bernoulli = Bernoulli::new(fill_ratio)
            .expect("bad fill ratio (should be between 0 - 1)")
            .sample_iter(rng)
            .take(ncell);
        // build grid from iterator
        let grid = Array::from_iter(bernoulli)
            .into_shape([height, width]).unwrap();
        Self {
            grid: grid,
            nstep: 0
        }
    }

    ///////////
    // Privates
    ///////////
    pub fn num_neighbors(&self, x:usize, y:usize) -> u32 {
        let (gridh, gridw) = self.grid.dim();
        // let slice = s![x-1..x+3, y-1..y+3];
        let xmin = (x).max(1) - 1;
        let ymin = (y).max(1) - 1;
        let xmax = (x+2).min(gridh);
        let ymax = (y+2).min(gridw);
        let cell_val = *self.get_cell(x, y) as u32;
        let neighbors = self.grid.slice(s![xmin..xmax, ymin..ymax]); // including cell (x,y)
        // dbg!(s![xmin..xmax, ymin..ymax]);
        // dbg!(&neighbors);
        neighbors.map(|&x| x as u32).sum() - cell_val
    }

    // TODO: return Option<&bool> instead of panicking
    pub fn get_cell(&self, x:usize, y:usize) -> &bool {
        self.grid.get((x,y)).expect("indices are out-of-bounds")
    }

    fn set_cell(&mut self, x:usize, y:usize, b:bool) {
        let ptr = self.grid.get_mut((x,y)).expect("indices are out-of-bounds");
        *ptr = b;
    }

    // cell state transition
    fn transition(live_cell:bool, n_neighbors:u32) -> bool {
        if live_cell {
            [2,3].contains(&n_neighbors)
        } else {
            n_neighbors == 3
        }
    }

    //////////
    // Publics
    //////////
    pub fn tick(&mut self) {
        // build array where (x,y) -> # of live neighbors
        let neighbors_iter = self.grid.indexed_iter()
            .map(|((x,y), _)| self.num_neighbors(x, y));
        let neighbors_grid = Array::from_iter(neighbors_iter)
            .into_shape(self.grid.raw_dim()).unwrap();

        // update each cell
        for ((x,y), c) in self.grid.indexed_iter_mut() {
            let newstate = Self::transition(*c, *neighbors_grid.get((x,y)).unwrap());
            *c = newstate;
        }

        // increment counter
        self.nstep += 1;
    }
    
}

// Pretty-print
impl fmt::Display for GameOfLife {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // print grid lines
        let print_lines: Vec<Vec<&str>> = self.grid.outer_iter()
            .map(|row| 
                row.iter().map(|&x|
                    match x {
                        false => "⬛️",
                        true => "⬜️"
                    }
                ).collect()
            )
            .collect();

        // write lines
        let print_lines_joined = print_lines.iter()
            .map(|chars| chars.join(""))
            .collect::<Vec<String>>()
            .join("\n")
            + "\n";
        write!(f, "=== STEP {} ===\n", self.nstep).unwrap();
        write!(f, "{}", print_lines_joined)
    }
}