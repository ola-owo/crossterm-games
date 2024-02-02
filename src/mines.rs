use std::fmt;

use itertools::izip;
use ndarray::{s,azip,Array,Array2,Zip};
use rand::{distributions::{Distribution,Bernoulli}, seq::SliceRandom};

use crate::Point;

// values to show on revealed non-mine squares
// const DIGIT_STRS: [&str; 9] = ["⬜️", "1", "2", "3", "4", "5", "6", "7", "8"];
const DIGIT_STRS: [&str; 9] = ["_", "1", "2", "3", "4", "5", "6", "7", "8"];
const HIDDEN_STR: &str = "#";
const MINE_STR: &str = "X";
const FLAG_STR: &str = "@";

pub enum SquareView {
    Hidden,
    Flag,
    Revealed(u32),
    Mine
}

#[derive(Debug, PartialEq)]

// move result returned from reveal()
pub enum MoveResult {
    Lose,
    Win,
    Ok,
    Err(String)
}

pub struct MineField {
    mines: Array2<bool>, // mines[i,j] == true if mine is at (i,j)
    neighbors: Array2<u32>, // neighbors[i,j] == # of neighboring mines
    revealed: Array2<bool>, // revealed[i,j] == true if (i,j) has been revealed
    flagged: Array2<bool>,  // flagged[i,j] == true if flag has been placed at (i,j)
    n_revealed: u32,
    dim: (usize, usize),
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
        let flagged = Array2::default(mines.raw_dim());
        let dim = mines.dim();

        Self {
            mines: mines,
            neighbors: neighbors,
            revealed: revealed,
            flagged: flagged,
            n_revealed: 0,
            dim: dim
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
        let flagged = Array2::default(mines.raw_dim());
        let dim = mines.dim();

        Self {
            mines: mines,
            neighbors: neighbors,
            revealed: revealed,
            flagged: flagged,
            n_revealed: 0,
            dim: dim
        }
    }


    ///////////
    // Privates
    ///////////

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

    fn neighbors_iter(&self, p: &Point) -> impl Iterator<Item=Point> {
        let (gridh, gridw) = self.dim;
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

        // let all_mines_flagged = Zip::from(&self.mines).and(&self.flagged)
        //     .all(|&m, &f| m == f);
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

    fn reveal_neighbors(&mut self, p: &Point) -> MoveResult {
        let mut res = MoveResult::Ok;
        for neighbor_pt in self.neighbors_iter(p) {
            if !self.is_revealed(&neighbor_pt).unwrap() {
                res = self.reveal(&neighbor_pt);
                if res != MoveResult::Ok {
                    break
                }
            }
        }
        res
    }

    fn chord(&mut self, p: &Point) -> MoveResult {
        let nn_mines: u32 = *self.neighbors.get(p.tuple()).unwrap();
        let nn_flags = self.neighbors_iter(p)
            .map(|p| self.is_flag(&p).unwrap() as u32)
            .sum();

        // only chord if # of neighboring flags == # of neighboring mines
        if nn_mines == nn_flags {
            self.reveal_neighbors(p)
        } else {
            MoveResult::Ok
        }
    }

    // reveal all mines after game is over
    fn reveal_all_mines(&mut self) {
        azip!((r in &mut self.revealed, &m in &self.mines)
            if m { *r = true }
        );
    }


    //////////
    // Publics
    //////////

    pub fn toggle_flag(&mut self, p: &Point) -> MoveResult {
        // if already revealed, do nothing
        if let Some(true) = self.is_revealed(p) {
            return MoveResult::Ok
        }

        // flip flagged state
        if let Some(flagged) = self.flagged.get_mut(p.tuple()) {
            *flagged = ! *flagged;
            MoveResult::Ok
        } else {
            MoveResult::Err(String::from("index OOB"))
        }
    }

    pub fn is_flag(&self, p: &Point) -> Option<bool> {
        self.flagged.get(p.tuple()).copied()
    }

    pub fn view_sq(&self, p: &Point) -> Option<SquareView> {
        let revealed = self.is_revealed(&p)?;
        let ismine = self.peek_mine(&p)?;
        let isflag = self.is_flag(&p)?;

        Some(match (revealed, ismine, isflag) {
            (false, _, false) => SquareView::Hidden,
            (false, _, true)  => SquareView::Flag,
            (true, false, _)  => SquareView::Revealed(*self.neighbors.get(p.tuple()).unwrap()),
            (true, true, _)   => SquareView::Mine
        })
    }

    // '_ is the anonymous lifetime of the ndarray iterators
    // + '_ indicates that iterator lifetime is bound by underlying ndarrays (I think)
    pub fn get_view_iter(&self) -> impl Iterator<Item=SquareView> + '_ {
        let sqdata_zip = izip!(
            self.revealed.iter(),
            self.mines.iter(),
            self.flagged.iter(),
            self.neighbors.iter()
        );

        sqdata_zip.map(|(&rev, &mine, &flag, &nn)| {
            match (rev, mine, flag, nn) {
                (false, _, false, _) => SquareView::Hidden,
                (false, _, true, _)  => SquareView::Flag,
                (true, false, _, nn) => SquareView::Revealed(nn),
                (true, true, _, _)   => SquareView::Mine
            }
        })
    }

    // reveal square (i,j)
    // (and all neighbors if square has no neighboring mines
    // Result indicates whether square is a mine
    //
    // possible outcomes:
    // 1 - hit a mine
    //   - show "you lose!" message
    //   - end game
    // 2 - non-mine; no non-mines left
    //   - show "you win!" message
    //   - end game
    // 3 - non-mine; nearby mines exist
    //   - reveal # of neighbors
    // 4 - non-mine; no nearby minees
    //   - reveal all neighbors recursively
    // 5 - OOB or already-revealed square
    //   - return Err without updating board
    pub fn reveal(&mut self, p: &Point) -> MoveResult {
        match self.view_sq(p) {
            None => return MoveResult::Err(String::from("index OOB")),
            Some(SquareView::Flag) => return MoveResult::Ok, // do nothing if flag
            Some(SquareView::Revealed(_)) => return self.chord(p),
            Some(SquareView::Hidden) => { // if hidden, mark square as revealed
                let rev = self.revealed.get_mut(p.tuple()).unwrap();
                *rev = true;
                self.n_revealed += 1;
            },
            _ => ()
        }

        // if a mine is hit, end game
        // (unless it's the 1st move)
        if *self.peek_mine(p).unwrap() {
            // if this is 1st move, move the mine
            if self.n_revealed == 1 {
                self.move_mine(p).unwrap();
            } else {
                self.reveal_all_mines();
                return MoveResult::Lose
            }
        }

        // if 0 neighbors, reveal all neighbors (recursively?)
        let nn = *self.neighbors.get(p.tuple()).unwrap();
        if nn == 0 {
            self.reveal_neighbors(p);
        }

        // check if game is won
        if self.game_won() {
            self.reveal_all_mines();
            MoveResult::Win
        } else {
            MoveResult::Ok
        }
    }
    
}

// Pretty-print
impl fmt::Display for MineField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // zip iterator of mines(bool), revealed(bool), and neighbors(u32)
        let sqdata_zip = Zip::from(&self.mines)
            .and(&self.revealed)
            .and(&self.neighbors)
            .and(&self.flagged);
        // print grid lines
        let print_lines = sqdata_zip.map_collect(|&mine, &rev, &nn, &flag| {
            match (mine, rev, nn, flag) {
                (_, false, _, false) => HIDDEN_STR,                // hidden square (⬛️)
                (_, false, _, true) => FLAG_STR, // space w/ nearby mines
                (true, true, _, _) => MINE_STR,                 // revealed mine
                (false, true, 0, _) => DIGIT_STRS[0],     // empty space
                (false, true, n, _) => DIGIT_STRS[n as usize], // space w/ nearby mines
            }
        });

        // write each (styled) character separately
        let ax_labeller = |i: usize| if i.rem_euclid(3)==0 {i.to_string()} else {"".to_string()};
        let mut write_res: fmt::Result = Ok(());
        for (i, row) in print_lines.outer_iter().enumerate() {
            // print vertical axis labels
            let v_ax_lbl = ax_labeller(i);
            write_res = write_res.and(write!(f, "{:2}", v_ax_lbl));

            // print board squares (double spaced)
            for chr in row.iter() {
                write_res = write_res.and(write!(f, "{} ", chr));
            }

            // double spacing between rows
            write_res = write_res.and(write!(f, "\n\n"));
        }

        // print horiz axis labels
        write_res = write_res.and(write!(f, "{:2}", ""));
        for h_ax_lbl in (0..self.mines.ncols()).map(ax_labeller) {
            write_res = write_res.and(write!(f, "{:2}", h_ax_lbl));
        }
        write_res = write_res.and(write!(f, "\n"));

        write_res
    }
}
