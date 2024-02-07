mod mines;
mod mineui;
mod point;

use std::io::{self, stdout, Write};
use std::fmt;

use crossterm::style::{ContentStyle, StyledContent, Stylize, Print};
use crossterm::terminal;
use crossterm::{queue, execute, cursor};

use mines::{MineField,MoveResult};
use mineui::{MineUI, MineUIAction, UIMode};

use crate::mines::SquareView;
use crate::point::Point;

const DIGIT_STRS: [&str; 9] = ["_", "1", "2", "3", "4", "5", "6", "7", "8"];
const HIDDEN_STR: &str = "#";
const MINE_STR: &str = "X";
const FLAG_STR: &str = "@";

pub struct MineSweeper {
    #[allow(dead_code)]
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
            print!("{}", self);
            
            // wait for input
            user_action = self.ui.wait_for_action_block().expect("failed to read input");
            
            match user_action {
                MineUIAction::Quit => break,
                MineUIAction::Help => self.print_help(&mut stdout()).unwrap_or(
                    self.message = self.fmt_err_msg("help-text failed".into())
                ),
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

    fn print_help<T: io::Write>(&self, f: &mut T) -> io::Result<()> {
        queue!(f,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            Print(mineui::HELP_TEXT)
        )?;
        self.ui.wait_for_action_block()?;
        Ok(())
    }
}

// Pretty-print
impl fmt::Display for MineSweeper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const ROW_SPACER: &str = "\r\n\r\n";
        const COL_SPACER: &str = " ";

        // reset terminal cursor
        execute!(stdout(), cursor::MoveTo(0,0), terminal::Clear(terminal::ClearType::All)).unwrap();

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
    let mut game = MineSweeper::new_beginner();
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)
        .expect("failed to enter alt screen");
    terminal::enable_raw_mode().unwrap();
    game.print_help(&mut stdout).expect("help-text failed");
    game.game_loop();
    execute!(stdout, Print("Press any key to exit ...")).unwrap();
    stdout.flush().unwrap();
    game.ui.wait_for_action_block().ok();
    terminal::disable_raw_mode().unwrap();
    queue!(stdout, terminal::LeaveAlternateScreen, cursor::Show)
        .expect("failed to exit alt screen");
}
