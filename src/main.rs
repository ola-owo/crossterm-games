mod mines;
mod mineui;

use std::io::{stdout, Write};
use std::fmt;

use crossterm::style::{ContentStyle, StyledContent, Stylize};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
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

    // Default beginner / intermediate / expert boards
    pub fn new_beginner() -> Self {
        Self::with_n_mines(8, 8, 10)
    }

    pub fn new_intermediate() -> Self {
        Self::with_n_mines(16, 16, 40)
    }

    pub fn new_expert() -> Self {
        Self::with_n_mines(16, 30, 99)
    }

    pub fn game_loop(&mut self) {
        let mut user_action: MineUIAction;
        loop {
            println!("{}", self);
            
            // wait for input
            user_action = self.ui.wait_for_action_block().expect("failed to read input");
            dbg!(&user_action);
            
            match user_action {
                MineUIAction::Quit => break,
                MineUIAction::Wait => {},
                MineUIAction::Mode(newmode) => self.ui.mode = newmode,
                MineUIAction::ToggleMode => self.ui.toggle_mode(),
                MineUIAction::Move(movedir) => {
                    self.message = "".to_string().reset();
                    self.ui.move_cursor(movedir).ok();
                },
                MineUIAction::Select => {
                    let p = self.ui.get_cursor();
                    let move_res = match self.ui.mode {
                        UIMode::Reveal => self.field.reveal(&p),
                        UIMode::Flag => self.field.toggle_flag(&p)
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
            MoveResult::Lose => {
                self.message = "You lose!".to_string().bold().white().on_dark_red();
                false
            },
            MoveResult::Win => {
                self.message = "You win!".to_string().bold().white().on_magenta();
                false
            },
            MoveResult::Err(ref msg) => {
                self.message = self.fmt_err_msg(msg.to_string());
                true
            }
            MoveResult::Ok => {
                self.message = "".to_string().reset();
                true
            }
        }
    }

    fn fmt_err_msg<D: fmt::Display + Stylize<Styled = StyledContent<D>>>(&mut self, msg: D) -> StyledContent<D> {
        msg.red()
    }

    // get a point (i,j)
    // this fxn mainly exists to make sure (i,j) is in-bounds
    fn get_pt(&self, i: usize, j: usize) -> Option<Point> {
        (i < self.gridh && j < self.gridw).then_some(Point {i, j})
    }
}

// Pretty-print
impl fmt::Display for MineSweeper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const ROW_SPACER: &str = "\r\n\r\n";
        const COL_SPACER: &str = " ";

        // reset terminal cursor
        execute!(stdout(), MoveTo(0,0), Clear(ClearType::All)).unwrap();

        let (cursor_i, cursor_j) = self.ui.get_cursor().tuple();
        let board_iter = self.field.get_view_iter();
        for (sq_ix, sq) in board_iter.enumerate() {
            // assign (styled) string for this square
            let mut sq_str = match sq {
                SquareView::Hidden => HIDDEN_STR.blue(),
                SquareView::Flag => FLAG_STR.dark_yellow(),
                SquareView::Mine => MINE_STR.red(),
                SquareView::Revealed(0) => DIGIT_STRS[0].dark_grey(),
                SquareView::Revealed(nn) => DIGIT_STRS[nn as usize].white()
            };

            // get coordinates of this square
            let sqi = sq_ix / self.gridw;
            let sqj = sq_ix.rem_euclid(self.gridw);

            // replace sq_str with cursor
            if sqi == cursor_i && sqj == cursor_j {
                sq_str = match self.ui.mode {
                    mineui::UIMode::Reveal => sq_str.bold().cyan(),
                    mineui::UIMode::Flag => sq_str.bold().yellow()
                }
            }

            // start new row
            if sqj == 0 {
                write!(f, "{ROW_SPACER}")?;
            }

            // draw square
            write!(f, "{sq_str}{COL_SPACER}")?;
        }

        // draw horizontal axis at the bottom
        write!(f, "{ROW_SPACER}")?;

        // print message
        write!(f, "{}\r\n", self.message)?;

        Ok(())
    }
}

fn main() {
    // println!("Hello, world!");
    let mut game = MineSweeper::new_beginner();
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alt screen");
    enable_raw_mode().unwrap();
    game.game_loop();
    print!("Press any key to exit ...");
    stdout().flush().unwrap();
    game.ui.wait_for_action_block().ok();
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen).expect("failed to exit alt screen");
}
