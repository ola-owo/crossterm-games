use std::io;

mod gameoflife;

use crate::gameoflife::GameOfLife;

fn main() {
    // println!("Hello, world!");
    let mut game = GameOfLife::random(40, 30, 0.3);
    print!("{}", game);
    for _ in 0..8 {
        game.tick();
        print!("{}", game);

        // Get input "(x,y)"
        // parse input
        // print value and NN of cell (x,y)
        println!("> ");
        let mut input_str = String::new();
        loop {
            input_str.clear();
            io::stdin()
                .read_line(&mut input_str)
                .expect("failed to read line");
            if input_str == "\n" {
                break
            }
            let input_vec: Vec<usize> = input_str.trim()
                .split(' ')
                .map(|x| x.parse::<usize>().expect("input format must be (x,y)") )
                .collect();
            let &input_x = input_vec.get(0).expect("input format must be (x,y)");
            let &input_y = input_vec.get(1).expect("input format must be (x,y)");
            let nn = game.num_neighbors(input_x, input_y);
            let cell = match game.get_cell(input_x, input_y) {
                false => "⬛️",
                true => "⬜️"
            };
            println!("cell ({},{}): {}", input_x, input_y, cell);
            println!("neighbors: {}", nn);
        }
    }
}
