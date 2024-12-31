use std::io;
use std::thread::sleep;
use std::time::Duration;

use crossterm::{
    cursor, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

mod gameoflife;
use gameoflife::GameOfLife;

fn main() {
    // go to alt screen and hide cursor
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide).unwrap();

    let mut game = GameOfLife::random(40, 30, 0.3);
    for _ in 0..200 {
        print!("{}", game);
        game.tick();
        sleep(Duration::from_secs_f32(0.1));
    }

    // go back to normal screen/cursor
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show).unwrap();
}
