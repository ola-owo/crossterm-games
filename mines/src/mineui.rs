use std::{io, time::Duration};

use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent};

use crate::Point;

pub const HELP_TEXT: &str = "
Use the arrow keys to move.\r
Press <space> to select the highlighted square.\r
Press <tab> to switch between reveal and flag mode.\r
Press <h> to show this help screen.\r
Press <q> to quit.\r
";

#[derive(Debug)]
pub enum MineUIAction {
    Wait,
    Move(MoveDirection),
    Mode(UIMode),
    ToggleMode,
    Select,
    Help,
    Quit,
}

#[derive(Debug)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum UIMode {
    Flag,
    Reveal,
}

pub struct MineUI {
    gridh: usize,
    gridw: usize,
    cursor: Point,
    pub mode: UIMode,
}

impl MineUI {
    /////////////
    // Statics //
    /////////////

    fn match_key_to_action(key_event: KeyEvent) -> MineUIAction {
        match key_event.code {
            KeyCode::Up => MineUIAction::Move(MoveDirection::Up),
            KeyCode::Down => MineUIAction::Move(MoveDirection::Down),
            KeyCode::Left => MineUIAction::Move(MoveDirection::Left),
            KeyCode::Right => MineUIAction::Move(MoveDirection::Right),
            KeyCode::Char(' ') => MineUIAction::Select,
            KeyCode::Char('f') => MineUIAction::Mode(UIMode::Flag),
            KeyCode::Char('r') => MineUIAction::Mode(UIMode::Reveal),
            KeyCode::Tab => MineUIAction::ToggleMode,
            KeyCode::Char('h') => MineUIAction::Help,
            KeyCode::Char('q') => MineUIAction::Quit,
            _ => MineUIAction::Wait,
        }
    }

    ///////////////////
    // Contstructors //
    ///////////////////

    pub fn new(height: usize, width: usize) -> Self {
        Self {
            gridh: height,
            gridw: width,
            cursor: Point::origin(),
            mode: UIMode::Reveal,
        }
    }

    /////////////
    // Publics //
    /////////////

    pub fn move_cursor(&mut self, dir: MoveDirection) -> Result<(), String> {
        let cur_i = self.cursor.0 as u32;
        let cur_j = self.cursor.1 as u32;

        let delta: (i32, i32) = match dir {
            MoveDirection::Up => (-1, 0),
            MoveDirection::Down => (1, 0),
            MoveDirection::Left => (0, -1),
            MoveDirection::Right => (0, 1),
        };

        // check upper and left boundaries
        let new_i = cur_i
            .checked_add_signed(delta.0)
            .ok_or("already at upper boundary")? as usize;
        let new_j = cur_j
            .checked_add_signed(delta.1)
            .ok_or("already at left boundary")? as usize;
        // check right and lower boundaries
        if new_i >= self.gridh {
            return Err("already at lower boundary".into());
        }
        if new_j >= self.gridw {
            return Err("already at rightward boundary".into());
        }

        // actually move
        self.reset_cursor(Point::new(new_i, new_j))
    }

    pub fn reset_cursor(&mut self, p: Point) -> Result<(), String> {
        if p.0 >= self.gridh || p.1 >= self.gridw {
            return Err(format!("point {} is OOB", p));
        }

        self.cursor = p;
        Ok(())
    }

    pub fn get_cursor(&self) -> Point {
        self.cursor
    }

    pub fn toggle_mode(&mut self) {
        let newmode = match self.mode {
            UIMode::Reveal => UIMode::Flag,
            UIMode::Flag => UIMode::Reveal,
        };
        self.mode = newmode;
    }

    // block until event happens
    pub fn wait_for_action_block(&self) -> io::Result<MineUIAction> {
        let action: MineUIAction;
        loop {
            let read_res = read();
            if let Key(key_event) = read_res? {
                action = Self::match_key_to_action(key_event);
                break;
            }
        }

        Ok(action)
    }

    // poll with a timeout
    #[allow(dead_code)]
    pub fn wait_for_action_poll(&self, timeout: u64) -> io::Result<MineUIAction> {
        let action: MineUIAction;
        let read_res = read();
        if poll(Duration::from_secs(timeout))? {
            // event happened
            if let Key(key_event) = read_res? {
                // keystroke event
                action = Self::match_key_to_action(key_event);
            } else {
                // non-keystroke event
                action = MineUIAction::Wait;
            }
        } else {
            // no event happened
            action = MineUIAction::Wait;
        }

        Ok(action)
    }
}
