use std::fmt;

use ndarray::{s,azip,Array,Array2,Zip};
use rand::{distributions::{Distribution,Bernoulli}, seq::SliceRandom};

// values to show on revealed non-mine squares
// const DIGIT_STRS: [&str; 9] = ["⬜️", "1", "2", "3", "4", "5", "6", "7", "8"];
const DIGIT_STRS: [&str; 9] = ["_", "1", "2", "3", "4", "5", "6", "7", "8"];
const HIDDEN_STR: &str = "#";
const MINE_STR: &str = "X";

#[derive(Debug)]

// move result from reveal()
pub enum MoveResult {
    LOSE,
    WIN,
    OK,
    ERR(String)
}

pub struct Point {
    i: usize,
    j: usize
}

impl Point {
    pub fn tuple(&self) -> (usize, usize) {
        (self.i, self.j)
    }

    pub fn arr(&self) -> [usize; 2] {
        [self.i, self.j]
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "({}, {})", self.i, self.j)
    }
}

pub struct MineField {
    mines: Array2<bool>, // mines[i,j] == true if mine is at (i,j)
    neighbors: Array2<u32>, // neighbors[i,j] == # of neighboring mines
    revealed: Array2<bool>, // revealed[i,j] == true if (i,j) has been revealed
    n_revealed: u32
}

impl MineField {
    //////////
    // Statics
    //////////

    // count how many neighboring mines each square has
    // only need to call this once when building the minefield
    fn n_neighbors_grid(mines: &Array2<bool>) -> Array2<u32> {
        // mines has size (M,N)
        // create copy of mines (as u32) with 1 layer of zero-padding
        let (gridh, gridw) = mines.dim(); // (M, N)
        let mut mines_pad: Array2<u32> = Array2::zeros((gridh+2, gridw+2)); // size (M+2, N+2)
        mines.mapv(|x| x as u32)
            .assign_to(mines_pad.slice_mut(s![1..-1, 1..-1]));

        // final array
        let mut nn: Array2<u32> = Array2::zeros(mines.raw_dim());

        // add up/down/left/right neighbors
        azip!((
            x in &mut nn,
            &d  in &mines_pad.slice(s![2..  , 1..-1]), // lower neighbors
            &u  in &mines_pad.slice(s![ ..-2, 1..-1]), // upper neighbors
            &r  in &mines_pad.slice(s![1..-1, 2..  ]), // right neighbors
            &l  in &mines_pad.slice(s![1..-1,  ..-2]), // left neighbors
        ) *x = d + u + l + r);

        // add diagonal neighbors
        azip!((
            x in &mut nn,
            &dr in &mines_pad.slice(s![2..  , 2..  ]), // lower-right neighbors
            &ur in &mines_pad.slice(s![ ..-2, 2..  ]), // upper-right neighbors
            &dl in &mines_pad.slice(s![2..  ,  ..-2]), // lower-left neighbors
            &ul in &mines_pad.slice(s![ ..-2,  ..-2])  // upper-left neighbors
        ) *x = *x + dr + dl + ur + ul);

        nn
    }


    ///////////////
    // Constructors
    ///////////////

    // build a minefield with a given # of mines
    pub fn with_n_mines(height: usize, width: usize, n_mines: usize) -> Self {
        // check inputs
        let n_cells = height * width;
        assert!(height > 0 && width > 0, "grid size must be non-zero!");
        assert!(n_mines < n_cells, "{}x{} grid can have up to {} mines!", height, width, n_cells);

        // build mine field
        // let mut rng = rand::thread_rng();
        let mine_ixs = rand::seq::index::sample(&mut rand::thread_rng(), n_cells, n_mines);
        let mut mines = Array2::default([height, width]);
        for ix in mine_ixs {
            let i = ix / width;
            let j = ix.rem_euclid(width);
            *mines.get_mut((i, j)).unwrap() = true;
        }

        // build other struct fields
        let revealed = Array2::default(mines.raw_dim());
        let neighbors = Self::n_neighbors_grid(&mines);

        Self {
            mines: mines,
            neighbors: neighbors,
            revealed: revealed,
            n_revealed: 0
        }
    }

    // build a minefield with a given ratio of mines
    #[allow(dead_code)]
    pub fn with_mine_ratio(height: usize, width: usize, fill_ratio: f64) -> Self {
        // check inputs
        let n_cells = height * width;
        assert!(height > 0 && width > 0, "grid size must be non-zero!");

        // build mine field
        let rng = rand::thread_rng();
        let bernoulli = Bernoulli::new(fill_ratio)
            .expect("bad fill ratio (should be between 0 - 1)")
            .sample_iter(rng)
            .take(n_cells);
        let mines = Array::from_iter(bernoulli)
            .into_shape([height, width]).unwrap();

        // build other struct fields
        let revealed = Array2::default(mines.raw_dim());
        let neighbors = Self::n_neighbors_grid(&mines);

        Self {
            mines: mines,
            neighbors: neighbors,
            revealed: revealed,
            n_revealed: 0
        }
    }


    ///////////
    // Privates
    ///////////

    // count how many mines are around square
    // fn num_neighbors(&self, i:usize, j:usize) -> u32 {
    //     let (gridh, gridw) = self.mines.dim();
    //     // let slice = s![x-1..x+3, y-1..y+3];
    //     let xmin = i.max(1) - 1;
    //     let ymin = j.max(1) - 1;
    //     let xmax = (i+2).min(gridh);
    //     let ymax = (j+2).min(gridw);
    //     let cell_val = *self.peek_mine(i, j).unwrap() as u32;
    //     let neighbors_slice = self.neighbors.slice(s![xmin..xmax, ymin..ymax]); // including cell (x,y)
    //     // dbg!(s![xmin..xmax, ymin..ymax]);
    //     // dbg!(&neighbors);
    //     neighbors_slice.map(|&x| x as u32).sum() - cell_val
    // }

    // check whether square has mine,
    // without fully revealing it
    fn peek_mine(&self, p: &Point) -> Option<&bool> {
        self.mines.get(p.tuple())
    }

    // check whether square is revealed
    fn is_revealed(&self, p: &Point) -> Option<bool> {
        match self.revealed.get(p.tuple()) {
            Some(x) => Some(x).copied(),
            None => None
        }
    }

    // iterate over neighbors
    // returns a 2-D Iter, or Err if (i,j) is out of bounds
    // fn iter_neighbors(&self, i: usize, j: usize) -> 
    // Result<Vec<(usize,usize)>, String> {
    //     let (gridh, gridw) = self.mines.dim();
    //     // check input coords
    //     if i >= gridh {
    //         return Err(format!("index i={i} is OOB"));
    //     } else if j >= gridw {
    //         return Err(format!("index j={j} is OOB"));
    //     }
    //     // get min/max i and j
    //     let imin = i.max(1) - 1;
    //     let jmin = j.max(1) - 1;
    //     let imax = (i+1).min(gridh-1);
    //     let jmax = (j+1).min(gridw-1);

    //     let iter = iter::zip(imin..=imax, jmin..=jmax)
    //         .filter(|&(a,b)| !(a==i && b==j));
    //     Ok(Vec::from_iter(iter))
    // }

    fn get_neighbors_iter(&self, p: &Point) -> impl Iterator<Item=Point> {
        let (gridh, gridw) = self.mines.dim();
        let i0 = p.i;
        let j0 = p.j;
        let imin = i0.max(1) - 1;
        let jmin = j0.max(1) - 1;
        let imax = (i0+1).min(gridh-1);
        let jmax = (j0+1).min(gridw-1);

        (imin..=imax).flat_map(move |i| {
            (jmin..=jmax).filter_map(move |j| {
                if i0==i && j0==j {None} else {Some(Point {i,j})}
            })
        })
    }

    // game is won if all non-mines have been revealed
    fn game_won(&self) -> bool {
        // zip(self.revealed.iter(), self.mines.iter())
        //     .all(|(&revealed, &mine)| {revealed || mine})
        let n_mines: u32 = self.mines.iter().map(|&x| x as u32).sum();
        let n_squares = self.mines.len() as u32;
        self.n_revealed == n_squares - n_mines
    }

    fn move_mine(&mut self, mine: &Point) -> Result<(), String> {
        // get reference to mine, throw error if not actually a mine
        let old_mine_ref = self.mines.get_mut(mine.tuple()).unwrap();
        if !*old_mine_ref {
            return Err(format!("{} is not a mine", &mine))
        }

        // pick a random non-mine square
        let mut rng = rand::thread_rng();
        loop {
            let square_ptr = self.mines.as_slice_mut()
                .expect("'mines' array is non-contiguous??")
                .choose_mut(&mut rng)
                .expect("'mines' is empty??");
            if !*square_ptr {
                // set random square as mine
                *square_ptr = true;
                break
            }
        }

        // unset old mine
        let old_mine_ref = self.mines.get_mut(mine.tuple()).unwrap();
        *old_mine_ref = false;

        // recompute num neighbors grid
        self.neighbors = Self::n_neighbors_grid(&self.mines);

        Ok(())
    }

    // reveal all mines after game is over
    fn reveal_all_mines(&mut self) {
        azip!((r in &mut self.revealed, &m in &self.mines) {
            if m { *r = true }
        });
    }


    //////////
    // Publics
    //////////

    // get a point (i,j)
    // this fxn mainly exists to make sure (i,j) is in-bounds
    pub fn get(&self, i: usize, j: usize) -> Option<Point> {
        let (gridh, gridw) = self.mines.dim();
        if i >= gridh || j >= gridw {
            None
        } else {
            Some(Point {i, j})
        }
    }

    // reveal square (i,j)
    // (and all neighbors if square has no neighboring mines
    // Result indicates whether square is a mine
    //
    /*
    possible outcomes:
        1- mine
            - show "you lose!" message
            - end game
        2- neighboring mines
            - reveal # of neighbors
        3- no neighboring minees
            - reveal all neighbors
    */
    pub fn reveal(&mut self, p: &Point) -> MoveResult {
        match self.revealed.get_mut(p.tuple()) {
            None => return MoveResult::ERR(String::from("index OOB")),
            Some(true) => return MoveResult::ERR(String::from("already revealed")),
            Some(r) => {
                *r = true;
                self.n_revealed += 1;
            }
        }

        // if a mine is hit, end game
        // (unless it's the 1st move)
        if *self.peek_mine(p).unwrap() {
            // if this is 1st move, move the mine
            if self.n_revealed == 1 {
                self.move_mine(p).unwrap();
            } else {
                self.reveal_all_mines();
                return MoveResult::LOSE
            }
        }

        // if 0 neighbors, reveal all neighbors (recursively?)
        let nn = *self.neighbors.get(p.tuple()).unwrap();
        if nn == 0 {
            // for (ii, jj) in self.iter_neighbors(i, j).unwrap() {
            for neighbor_pt in self.get_neighbors_iter(p) {
                if !self.is_revealed(&neighbor_pt).unwrap() {
                    self.reveal(&neighbor_pt);
                }
            }
        }

        // check if game is won
        if self.game_won() {
            self.reveal_all_mines();
            MoveResult::WIN
        } else {
            MoveResult::OK
        }
    }
    
}

// Pretty-print
impl fmt::Display for MineField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // zip iterator of mines(bool), revealed(bool), and neighbors(u32)
        let mrn_zip = Zip::from(&self.mines)
            .and(&self.revealed)
            .and(&self.neighbors);
        // print grid lines
        let print_lines = mrn_zip.map_collect(|&m, &r, &n| {
            match (m,r,n) {
                (_, false, _) => HIDDEN_STR,   // hidden square (⬛️)
                (true, true, _) => MINE_STR,      // revealed mine
                (false, true, n) => DIGIT_STRS[n as usize]
            }
        });
        let print_lines_joined = print_lines.outer_iter().map(|line| {
            String::from_iter(line.iter().map(|&x| x))
        })
            .collect::<Vec<String>>()
            .join("\n");

        writeln!(f, "{}", print_lines_joined)
    }
}
