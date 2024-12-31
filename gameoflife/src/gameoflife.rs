use std::{fmt, io};

use crossterm::queue;
use crossterm::cursor::MoveTo;
use crossterm::terminal::{Clear, ClearType};
use ndarray::{azip, s, Array, Array2};
use rand::distributions::{Bernoulli, Distribution};

/// Game of Life state (grid and step counter)
pub struct GameOfLife {
    grid: Array2<bool>,
    nstep: u32,
}

impl GameOfLife {
    //////////////////
    // Constructors //
    //////////////////

    /// Make a randomized grid with a specified ratio of active cells
    pub fn random(height: usize, width: usize, fill_ratio: f64) -> Self {
        // ncell = number of cells in grid
        let ncell = height * width;

        // make bernoulli iterator, capped at [ncell] values
        let rng = rand::thread_rng();
        let bernoulli = Bernoulli::new(fill_ratio)
            .expect("bad fill ratio (should be between 0 - 1)")
            .sample_iter(rng)
            .take(ncell);

        // build grid from iterator
        let grid = Array::from_iter(bernoulli)
            .into_shape([height, width])
            .unwrap();
        Self {
            grid: grid,
            nstep: 0,
        }
    }

    /////////////
    // Publics //
    /////////////

    /// Move forward one time-step
    pub fn tick(&mut self) {
        // build array where (x,y) -> # of live neighbors
        let neighbors_grid = self.num_neighbors_grid();

        // update each cell
        for ((x, y), c) in self.grid.indexed_iter_mut() {
            let newstate = Self::transition(*c, *neighbors_grid.get((x, y)).unwrap());
            *c = newstate;
        }

        // increment counter
        self.nstep += 1;
    }

    //////////////
    // Privates //
    //////////////

    /// cell state transition function
    fn transition(live_cell: bool, n_neighbors: u32) -> bool {
        if live_cell {
            [2, 3].contains(&n_neighbors)
        } else {
            n_neighbors == 3
        }
    }

    /// get each cell's number of neighbors
    fn num_neighbors_grid(&self) -> Array2<u32> {
        // create copy of grid (as u32) with 1 layer of zero-padding
        let (gridh, gridw) = self.grid.dim();
        let mut grid_pad: Array2<u32> = Array2::zeros((gridh + 2, gridw + 2));
        self.grid
            .mapv(|x| x as u32)
            .assign_to(grid_pad.slice_mut(s![1..-1, 1..-1]));

        // num-neighbors array
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
}

impl fmt::Display for GameOfLife {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // clear screen
        queue!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).expect("display e");

        // make grid lines
        let print_lines: Vec<Vec<&str>> = self
            .grid
            .outer_iter()
            .map(|row| row.iter().map(|&x| if x { "⬛️" } else { "⬜️" }).collect())
            .collect();

        // write lines
        let print_lines_joined = print_lines
            .iter()
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
