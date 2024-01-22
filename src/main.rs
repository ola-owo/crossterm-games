mod mines;

use std::{io, num::ParseIntError};
use std::io::Write;

use mines::{MineField,MoveResult,Point};


fn main() {
    // println!("Hello, world!");
    let mut game = MineField::with_n_mines(8, 9, 30);
    game_loop(&mut game);
}

fn game_loop(game: &mut MineField) {
    let mut input_str = String::new();
    let mut input_pt: Point;
    loop {
        print!("{}", game);

        // Get input "x y" and parse it into a Point
        loop {
            print!("> ");
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
            if input_res_vec.iter().any(|x| x.is_err()) 
            || input_res_vec.len() != 2 {
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
            match input_pt_opt {
                None => {
                    println!("input ({},{}) is OOB", input_x, input_y);
                    continue
                },
                Some(pt) => input_pt = pt
            }
            break
        }

        // reveal the square
        let reveal_res = game.reveal(&input_pt);
        match reveal_res {
            MoveResult::LOSE => {
                print!("{}", game);
                println!("You lose!");
                break
            },
            MoveResult::WIN => {
                print!("{}", game);
                println!("You win!");
                break
            },
            MoveResult::ERR(ref msg) => println!("bad input: {}", &msg),
            MoveResult::OK => ()
        }
        dbg!(&reveal_res);
    }
}
