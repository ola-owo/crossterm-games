use std::{fmt, io};

use ndarray::{s,azip,Array,Array2};
use rand::distributions::{Distribution,Bernoulli};
use crossterm::terminal::{Clear,ClearType};
use crossterm::cursor::MoveTo;
use crossterm::queue;

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

    // get the value of a cell
    // true = live, false = dead
    pub fn get_cell(&self, x:usize, y:usize) -> Option<&bool> {
        self.grid.get((x,y))
    }

    ///////////
    // Privates
    ///////////
    fn num_neighbors(&self, x:usize, y:usize) -> u32 {
        let (gridh, gridw) = self.grid.dim();
        // let slice = s![x-1..x+3, y-1..y+3];
        let xmin = (x).max(1) - 1;
        let ymin = (y).max(1) - 1;
        let xmax = (x+2).min(gridh);
        let ymax = (y+2).min(gridw);
        let cell_val = *self.get_cell(x, y).unwrap() as u32;
        let neighbors = self.grid.slice(s![xmin..xmax, ymin..ymax]); // including cell (x,y)
        // dbg!(s![xmin..xmax, ymin..ymax]);
        // dbg!(&neighbors);
        neighbors.map(|&x| x as u32).sum() - cell_val
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
    
    // (EXPERIMENTAL) vectorized num. neighbors
    fn num_neighbors_grid(&self) -> Array2<u32> {
        // grid has size (M,N)
        // create copy of grid (as u32) with 1 layer of zero-padding
        let (gridh, gridw) = self.grid.dim(); // (M, N)
        let mut grid_pad: Array2<u32> = Array2::zeros((gridh+2, gridw+2)); // size (M+2, N+2)
        self.grid.mapv(|x| x as u32)
            .assign_to(grid_pad.slice_mut(s![1..-1, 1..-1]));

        // final array
        let mut nn: Array2<u32> = Array2::zeros(self.grid.raw_dim());

        // add up/down/left/right neighbors
        azip!((
            x in &mut nn,
            &d  in &grid_pad.slice(s![2..  , 1..-1]), // lower neighbors
            &u  in &grid_pad.slice(s![ ..-2, 1..-1]), // upper neighbors
            &r  in &grid_pad.slice(s![1..-1, 2..  ]), // right neighbors
            &l  in &grid_pad.slice(s![1..-1,  ..-2]), // left neighbors
        ) *x = d + u + l + r);

        // add diagonal neighbors
        azip!((
            x in &mut nn,
            &dr in &grid_pad.slice(s![2..  , 2..  ]), // lower-right neighbors
            &ur in &grid_pad.slice(s![ ..-2, 2..  ]), // upper-right neighbors
            &dl in &grid_pad.slice(s![2..  ,  ..-2]), // lower-left neighbors
            &ul in &grid_pad.slice(s![ ..-2,  ..-2])  // upper-left neighbors
        ) *x = *x + dr + dl + ur + ul);

        nn
    }

    //////////
    // Publics
    //////////
    pub fn tick(&mut self) {
        // build array where (x,y) -> # of live neighbors
        let neighbors_grid = self.num_neighbors_grid();

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
        // clear screen
        queue!(io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0)
        ).expect("display e");

        // make grid lines
        let print_lines: Vec<Vec<&str>> = self.grid.outer_iter()
            .map(|row| 
                row.iter().map(|&x| if x {"⬛️"} else {"⬜️"}).collect()
            )
            .collect();

        // write lines
        let print_lines_joined = print_lines.iter()
            .map(|chars| chars.join(""))
            .collect::<Vec<String>>()
            .join("\n")
            + "\n";
        writeln!(f, "{}", print_lines_joined)?;

        // status bar
        writeln!(f)?;
        writeln!(f, "=== STEP {} ===\n", self.nstep)?;

        Ok(())
    }
}

