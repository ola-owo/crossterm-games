use std::fmt;

use ndarray::{azip, Array, Array1, Array2};

struct Langton {
    grid: Grid,
    ant: Ant,
    nstep: u32,
}

impl Langton {
    pub fn new_centered(height: usize, width: usize) -> Self {
        Self {
            grid: Grid::new(height, width),
            ant: Ant {
                pos: Array1::from(vec![height / 2, width / 2]),
                vel: Direction::new(0, 1),
            },
            nstep: 0,
        }
    }

    fn move_ant(&mut self) {
        let mut pos = self.ant.pos.mapv(|x| x as i32);
        let vel = self.ant.vel.vec.mapv(|x| x as i32);
        azip!((p in &mut pos, &v in &vel, g in self.grid.data.shape()) *p = (*p + v).rem_euclid(*g as i32));
        self.ant.pos = pos.mapv(|x| x as usize);
    }

    fn rotate_ant(&mut self, rot: RotationDirection) {
        self.ant.rotate(rot);
    }

    fn get_square_ptr_mut(&mut self) -> &mut bool {
        let &ix: &[usize; 2] = &self.ant.get_pos();
        self.grid
            .data
            .get_mut(ix)
            .expect("ant position is out of bounds")
    }

    fn get_square_ptr(&self) -> &bool {
        let &ix: &[usize; 2] = &self.ant.get_pos();
        self.grid
            .data
            .get(ix)
            .expect("ant position is out of bounds")
    }

    fn flip_square(ptr: &mut bool) {
        *ptr = !(*ptr);
    }

    pub fn tick(&mut self) {
        // get pointer to grid square, rotate ant
        let rot = match *self.get_square_ptr() {
            false => RotationDirection::CW,
            true => RotationDirection::CCW,
        };
        self.rotate_ant(rot);
        // get mutable pointer to grid square, flip squre
        Langton::flip_square(self.get_square_ptr_mut());
        // move ant
        self.move_ant();
        // increment step counter
        self.nstep += 1;
    }
}

// Pretty-print grid + ant
impl fmt::Display for Langton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // print grid
        let mut print_lines: Vec<Vec<String>> = self
            .grid
            .data
            .outer_iter()
            .map(|row| {
                row.iter()
                    .map(|&x| match x {
                        false => String::from("⬛️"),
                        true => String::from("⬜️"),
                    })
                    .collect()
            })
            .collect();

        // overlay ant
        let ant_icon = match self.ant.vel.vec.as_slice().unwrap() {
            &[0, 1] => "➡️",
            &[1, 0] => "⬇",
            &[0, -1] => "⬅️",
            &[-1, 0] => "⬆️",
            // ➡️⬇️⬅️⬆️
            // →↓←↑
            // 🟥🟠⭕
            _ => panic!("unknown ant direction"),
        };
        let ant_pos = self.ant.get_pos();
        let z = print_lines[ant_pos[0]]
            .iter_mut()
            .nth(ant_pos[1])
            .expect("ant is out-of-bounds");
        write!(
            f,
            "ant is at position ({},{}): {} {}\n",
            &ant_pos[0], &ant_pos[1], &z, ant_icon
        )
        .unwrap();
        *z = String::from(ant_icon);

        // write lines
        let print_lines_joined = print_lines
            .iter()
            .map(|chars| chars.join(""))
            .collect::<Vec<String>>()
            .join("\n")
            + "\n";
        write!(f, "{}", print_lines_joined)
    }
}

struct Grid {
    data: Array2<bool>,
}

impl Grid {
    //////////////////
    // Constructors //
    //////////////////
    pub fn new(height: usize, width: usize) -> Grid {
        Grid {
            data: Array2::<bool>::default((height, width)),
        }
    }
}

enum RotationDirection {
    CW,
    CCW,
}

#[derive(Debug)]
struct Direction {
    vec: Array1<i32>,
}

impl Direction {
    pub fn new(dx: i32, dy: i32) -> Direction {
        Direction {
            vec: Array::from_vec(vec![dx, dy]),
        }
    }

    pub fn rotate(&mut self, rot: RotationDirection) {
        let rot_mat = match rot {
            RotationDirection::CW => Array2::from_shape_vec((2, 2), vec![0, 1, -1, 0]),
            RotationDirection::CCW => Array2::from_shape_vec((2, 2), vec![0, -1, 1, 0]),
        }
        .unwrap();
        self.vec = self.vec.dot(&rot_mat);
    }
}

#[derive(Debug)]
struct Ant {
    pos: Array1<usize>,
    vel: Direction,
}

impl Ant {
    fn rotate(&mut self, rot: RotationDirection) {
        self.vel.rotate(rot);
    }

    pub fn get_pos(&self) -> [usize; 2] {
        self.pos
            .as_slice()
            .unwrap()
            .try_into()
            .expect("invalid position vector")
    }
}

//impl Ant {

fn main() {
    println!("Hello, world!");
    const GRID_X: usize = 40;
    const GRID_Y: usize = 50;
    let mut langton = Langton::new_centered(GRID_X, GRID_Y);

    print!("{}", langton);
    for _ in 0..3000 {
        // dbg!("{}", &langton.ant);
        langton.tick();
    }
    print!("{}", langton);
}
