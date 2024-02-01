mod mines;
mod mineui;

use std::{io, num::ParseIntError};
use std::io::Write;
use std::fmt;

use crossterm::style::{ContentStyle, StyledContent, Stylize};
use crossterm::terminal::{Clear,ClearType};
use crossterm::cursor::MoveTo;
use crossterm::execute;

use mines::{MineField,MoveResult};
use mineui::{MineUI, MineUIAction, UIMode};

use crate::mines::SquareView;

const DIGIT_STRS: [&str; 9] = ["_", "1", "2", "3", "4", "5", "6", "7", "8"];
const HIDDEN_STR: &str = "#";
const MINE_STR: &str = "X";
const FLAG_STR: &str = "@";

#[derive(Clone, Copy)]
pub struct Point {
    i: usize,
    j: usize
}

impl Point {
    pub fn origin() -> Self {
        Self {i:0, j:0}
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

pub struct MineSweeper {
    gridh: usize,
    gridw: usize,
    field: MineField,
    ui: MineUI,
    message: StyledContent<String>
}

impl MineSweeper {
    pub fn with_n_mines(height: usize, width: usize, n_mines: usize) -> Self {
        Self {
            gridh: height,
            gridw: width,
            field: MineField::with_n_mines(height, width, n_mines),
            ui: MineUI::new(height, width),
            message: StyledContent::new(ContentStyle::default(), "".into())
        }
    }

    pub fn with_mine_ratio(height: usize, width: usize, fill_ratio: f64) -> Self {
        Self {
            gridh: height,
            gridw: width,
            field: MineField::with_mine_ratio(height, width, fill_ratio),
            ui: MineUI::new(height, width),
            message: StyledContent::new(ContentStyle::default(), "".into())
        }
    }

    fn game_loop(&mut self) {
        let mut user_action: MineUIAction;
        loop {
            println!("{}", self);

            // wait for input
            user_action = self.ui.wait_for_action_block().expect("failed to read input");
            dbg!(&user_action);

            match user_action {
                MineUIAction::QUIT => break,
                MineUIAction::WAIT => {},
                MineUIAction::MODE(newmode) => self.ui.mode = newmode,
                MineUIAction::MOVE(movedir) => {
                    self.message = "".to_string().reset();
                    self.ui.move_cursor(movedir)
                        .unwrap_or_else(|e| self.message = self.fmt_err_msg(e));
                },
                MineUIAction::SELECT => {
                    let p = self.ui.get_cursor();
                    let move_res = match self.ui.mode {
                        UIMode::REVEAL => self.field.reveal(&p),
                        UIMode::FLAG => self.field.flag(&p)
                    };
                    if !self.handle_res(&move_res) {
                        println!("{}", self);
                        break
                    }
                },
            }
        }
    }

    // output indicates whether to keep looping
    fn handle_res(&mut self, res: &MoveResult) -> bool {
        match res {
            MoveResult::LOSE => {
                self.message = "You lose!".to_string().bold().white().on_dark_red();
                false
            },
            MoveResult::WIN => {
                self.message = "You win!".to_string().bold().white().on_magenta();
                false
            },
            MoveResult::ERR(ref msg) => {
                self.message = self.fmt_err_msg(format!("bad input: {}", &msg).to_string());
                true
            }
            MoveResult::OK => {
                self.message = "".to_string().reset();
                true
            }
        }
    }

    fn fmt_err_msg<D: fmt::Display + Stylize<Styled = StyledContent<D>>>(&mut self, msg: D) -> StyledContent<D> {
        msg.red()
    }
}

// Pretty-print
impl fmt::Display for MineSweeper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const ROW_SPACER: &str = "\r\n\r\n";
        const COL_SPACER: &str = " ";
        let ax_labeller = |i: usize| if i.rem_euclid(3)==0 {i.to_string()} else {"".to_string()};

        // reset terminal cursor
        execute!(io::stdout(), MoveTo(0,0), Clear(ClearType::All)).unwrap();

        let (cursor_i, cursor_j) = self.ui.get_cursor().tuple();
        let board_iter = self.field.get_view_iter();
        for (sq_ix, sq) in board_iter.enumerate() {
            // assign (styled) string for this square
            let mut sq_str = match sq {
                SquareView::HIDDEN => HIDDEN_STR.blue(),
                SquareView::FLAG => FLAG_STR.dark_yellow(),
                SquareView::MINE => MINE_STR.red(),
                SquareView::REVEALED(0) => DIGIT_STRS[0].dark_grey(),
                SquareView::REVEALED(nn) => DIGIT_STRS[nn as usize].white()
            };

            // get coordinates of this square
            let sqi = sq_ix / self.gridw;
            let sqj = sq_ix.rem_euclid(self.gridw);

            // replace sq_str with cursor
            if sqi == cursor_i && sqj == cursor_j {
                sq_str = match self.ui.mode {
                    mineui::UIMode::REVEAL => sq_str.bold().cyan(),
                    mineui::UIMode::FLAG => sq_str.bold().yellow()
                }
            }

            // start new row
            if sqj == 0 {
                let v_ax_lbl = ax_labeller(sqi);
                write!(f, "{ROW_SPACER}")?;
                write!(f, "{v_ax_lbl:2}")?;
            }

            // draw square
            write!(f, "{sq_str}{COL_SPACER}")?;
        }

        // draw horizontal axis at the bottom
        write!(f, "{ROW_SPACER}")?;
        write!(f, "{:2}", "")?;
        for h_ax_lbl in (0..self.gridw).map(ax_labeller) {
            write!(f, "{h_ax_lbl:2}")?;
        }
        write!(f, "{ROW_SPACER}")?;

        // print message
        write!(f, "{}\r\n", self.message)?;

        Ok(())
    }
}

fn main() {
    // println!("Hello, world!");
    let mut game = MineSweeper::with_n_mines(8, 9, 10);
    game.game_loop();
}

fn get_input(game: &MineField) -> Point {
    let mut input_str = String::new();
    loop {
        print!("\n> ");
        io::stdout().flush().expect("couldn't flush output");
        input_str.clear();
        io::stdin()
            .read_line(&mut input_str)
            .expect("failed to read line");

        // parse input
        let input_res_vec: Vec<Result<usize, ParseIntError>> = input_str.trim()
            .split(' ')
            .map(|x| x.parse::<usize>())
            .collect();
        if input_res_vec.iter().any(|x| x.is_err()) || input_res_vec.len() != 2 {
            println!("input format must be 'x y'");
            continue
        }
        let input_vec: Vec<usize> = input_res_vec.into_iter().map(|x| x.unwrap()).collect();
        if input_vec.len() != 2 {
            println!("input format must be 'x y'");
            continue
        }
        let input_x = *input_vec.get(0).expect("input format must be 'x y'");
        let input_y = *input_vec.get(1).expect("input format must be 'x y'");
        let input_pt_opt = game.get(input_x, input_y);
        if let Some(input_pt) = input_pt_opt {
            return input_pt
        } else {
            println!("input ({},{}) is OOB", input_x, input_y);
            continue
        }
    }
}