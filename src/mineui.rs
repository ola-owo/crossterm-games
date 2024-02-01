use std::{io, time::Duration};

use crossterm::event::{poll, read, Event::Key, KeyCode, KeyEvent};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

use crate::Point;

#[derive(Debug)]
pub enum MineUIAction {
    WAIT,
    MOVE(MoveDirection),
    MODE(UIMode),
    SELECT,
    QUIT
}

#[derive(Debug)]
pub enum MoveDirection {
    UP,
    DOWN,
    LEFT,
    RIGHT
}

#[derive(Debug)]
pub enum UIMode {
    FLAG,
    REVEAL
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
            KeyCode::Up => MineUIAction::MOVE(MoveDirection::UP),
            KeyCode::Down => MineUIAction::MOVE(MoveDirection::DOWN),
            KeyCode::Left => MineUIAction::MOVE(MoveDirection::LEFT),
            KeyCode::Right => MineUIAction::MOVE(MoveDirection::RIGHT),
            KeyCode::Enter => MineUIAction::SELECT,
            KeyCode::Char('f') => MineUIAction::MODE(UIMode::FLAG),
            KeyCode::Char('r') => MineUIAction::MODE(UIMode::REVEAL),
            KeyCode::Char('q') => MineUIAction::QUIT,
            _ => MineUIAction::WAIT
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
            mode: UIMode::REVEAL
        }
    }

    /////////////
    // Publics //
    /////////////

    pub fn move_cursor(&mut self, dir: MoveDirection) -> Result<(),String> {
        let cur_i = self.cursor.tuple().0 as u32;
        let cur_j = self.cursor.tuple().1 as u32;

        
        let delta: (i32, i32) = match dir {
            MoveDirection::UP => (-1, 0),
            MoveDirection::DOWN => (1, 0),
            MoveDirection::LEFT => (0, -1),
            MoveDirection::RIGHT => (0, 1)
        };
        
        // check upper and left boundaries
        let new_i = cur_i.checked_add_signed(delta.0)
            .ok_or("already at upper boundary")?
            as usize;
        let new_j = cur_j.checked_add_signed(delta.1)
            .ok_or("already at left boundary")?
            as usize;
        // check right and lower boundaries
        if new_i >= self.gridh {
            return Err("already at lower boundary".into())
        }
        if new_j >= self.gridw {
            return Err("already at rightward boundary".into())
        }

        // actually move
        self.reset_cursor(Point {i: new_i, j: new_j})

    }

    pub fn reset_cursor(&mut self, p: Point) -> Result<(),String> {
        let (pi, pj) = p.tuple();
        if pi >= self.gridh || pj >= self.gridw {
            return Err(format!("point {} is OOB", p));
        }

        self.cursor = p;
        Ok(())
    }

    pub fn get_cursor(&self) -> Point {
        self.cursor
    }

    // block until event happens
    pub fn wait_for_action_block(&self) -> io::Result<MineUIAction> {
        // enable_raw_mode();
        let action: MineUIAction;
        loop {
            enable_raw_mode()?;
            let read_res = read();
            disable_raw_mode()?;
            if let Key(key_event) = read_res? {
                action = Self::match_key_to_action(key_event);
                break
            }
        }

        Ok(action)
    }

    // poll with a timeout
    pub fn wait_for_action_poll(&self, timeout: u64) -> io::Result<MineUIAction> {
        let action: MineUIAction;
        enable_raw_mode()?;
        let read_res = read();
        disable_raw_mode()?;
        if poll(Duration::from_secs(timeout))? {
            // event happened
            if let Key(key_event) = read_res? {
                // event was a keystroke
                action = Self::match_key_to_action(key_event);
            } else {
                // non-keystroke event
                action = MineUIAction::WAIT;
            }
        } else {
            // no event happened
            action = MineUIAction::WAIT;
        }

        Ok(action)
    }
}