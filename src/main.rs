use std::collections::{HashSet, VecDeque};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use clap::Parser;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use itertools::Itertools;

#[derive(Parser, Debug)]
#[command(about = "Formatting: '-' uncovered, '?' covered, 'X' known bomb, [1 - 9] numbers")]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long, help = "Shows individual board possibilities instead of only the guaranteed cells")]
    show_possibilities: bool,
}

fn main() {
    let args = Args::parse();

    let data = if let Ok(data) = fs::read_to_string(args.file) {
        data
    } else {
        println!("Failed to read input file");
        return;
    };

    let initial_board = match Board::from_string(data) {
        Ok(board) => board,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    println!("Input:\n{initial_board}\n");

    match initial_board.validate_board() {
        Ok(remaining) => {
            if remaining == 0 {
                println!("Board already solved");
                return;
            }
        }
        Err(e) => { println!("Invalid board:\n\t{e}"); return; }
    }

    let mut board = initial_board.clone();

    while board.complete_solvable() {}
    println!("After static analysis:\n{board}\n");

    let mut open_boards = VecDeque::new();
    let mut visited = HashSet::new();
    let mut possibilities = Vec::new();
    let mut ignore = HashSet::new();

    let mut first = true;
    while !open_boards.is_empty() || first {
        let board = if !first {
            open_boards.pop_front().unwrap()
        }
        else {
            first = false;
            board.clone()
        };

        let (solved_boards, new_open_boards) = board.get_possible_boards(&mut visited, &mut ignore);

        for board in solved_boards {
            if args.show_possibilities {
                println!("Possible board found:\n{board}\n");
            }
            possibilities.push(board);
        }

        for board in new_open_boards {
            open_boards.push_back(board);
        }
    }

    println!("Finished finding solutions - {} possibilities\n", possibilities.len());

    let guaranteed = Board::compile_guaranteed(&initial_board, &possibilities, &ignore);
    println!("{guaranteed}");

    if !args.show_possibilities {
        println!("Use the -s argument to show individual board possibilities");
    }
}

#[derive(Debug, Copy, Clone)]
enum CellTypes {
    Covered,
    Bomb,
    Value(u8),
}

impl CellTypes {
    pub fn from_char(input: char) -> Result<CellTypes, String> {
        match input {
            '-' => Ok(CellTypes::Value(0)),
            '?' => Ok(CellTypes::Covered),
            'X' | 'x' => Ok(CellTypes::Bomb),
            '1' => Ok(CellTypes::Value(1)),
            '2' => Ok(CellTypes::Value(2)),
            '3' => Ok(CellTypes::Value(3)),
            '4' => Ok(CellTypes::Value(4)),
            '5' => Ok(CellTypes::Value(5)),
            '6' => Ok(CellTypes::Value(6)),
            '7' => Ok(CellTypes::Value(7)),
            '8' => Ok(CellTypes::Value(8)),
            c => Err(format!("Unrecognised character '{}'", c.escape_debug())),
        }
    }

    pub fn char(&self) -> char {
        /*match &self {
            CellTypes::Covered => '◼',
            CellTypes::Bomb => 'x',
            CellTypes::Value(v) =>  {
                if *v == 0 {
                    '◻'
                }
                else {
                    v.to_string().chars().next().unwrap()
                }
            }
        }*/
        match &self {
            CellTypes::Covered => '?',
            CellTypes::Bomb => 'x',
            CellTypes::Value(v) =>  {
                if *v == 0 {
                    '-'
                }
                else {
                    v.to_string().chars().next().unwrap()
                }
            }
        }
    }

    pub fn id(&self) -> u8 {
        match &self {
            CellTypes::Value(v) => *v,
            CellTypes::Covered => 9,
            CellTypes::Bomb => 10
        }
    }
}

#[derive(Debug, Clone)]
struct Board {
    board: Vec<Vec<CellTypes>>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn compile_guaranteed(base: &Board, possibilities: &[Board], ignore: &HashSet<(usize, usize)>) -> String {
        let mut board = Vec::with_capacity(base.height);
        for _ in 0..base.height {
            let mut line = Vec::with_capacity(base.width);
            for _ in 0..base.width {
                // Bomb, Not
                line.push((0usize, 0usize));
            }
            board.push(line);
        }

        for possibility in possibilities {
            for x in 0..base.width {
                for y in 0..base.height {
                    let is_bomb = match &possibility.board[y][x] {
                        CellTypes::Value(_) => continue,
                        CellTypes::Bomb => true,
                        CellTypes::Covered => false,
                    };

                    if is_bomb {
                        board[y][x] = (board[y][x].0 + 1, board[y][x].1);
                    }
                    else {
                        board[y][x] = (board[y][x].0, board[y][x].1 + 1);
                    }
                }
            }
        }

        let mut board_probabilities = Vec::with_capacity(base.height);
        let mut output = Vec::with_capacity(base.height);
        let mut found = false;

        for y in 0..base.height {
            output.push(Vec::with_capacity(base.width));
            board_probabilities.push(Vec::with_capacity(base.width));
            for x in 0..base.width {
                board_probabilities[y].push(0f64);
                if board[y][x].0 == 0 && board[y][x].1 == 0 || ignore.contains(&(x, y)) || (board[y][x].0 > 0 && board[y][x].1 == 0 && matches!(base.board[y][x], CellTypes::Bomb)) {
                    output[y].push(base.board[y][x].char());
                }
                else if board[y][x].0 > 0 && board[y][x].1 == 0 {
                    output[y].push('#');
                    found = true;
                }
                else if board[y][x].0 == 0 && board[y][x].1 > 0 {
                    output[y].push('O');
                    found = true;
                }
                else {
                    if !found {
                        board_probabilities[y][x] = (board[y][x].1 as f64) / (board[y][x].1 as f64 + board[y][x].0 as f64);
                    }

                    output[y].push('?');
                }
            }
        }


        if found {
            let output_string = output.into_iter().map(|line| line.into_iter().collect::<String>()).join("\n");
            format!("Guaranteed cells:\n{output_string}\n\nKey:\n\t'#' - Guaranteed bomb\n\t'O' - Guaranteed safe\n")
        }
        else {
            let mut max: Option<(f64, (usize, usize))> = None;
            for y in 0..base.height {
                for x in 0..base.width {
                    if max.is_none() || board_probabilities[y][x] > max.unwrap().0 {
                        max = Some((board_probabilities[y][x], (x, y)));
                    }
                }
            }

            let (x, y) = max.unwrap().1;
            let probability = max.unwrap().0;

            output[y][x] = '@';
            let output_string = output.into_iter().map(|line| line.into_iter().collect::<String>()).join("\n");

            format!("{output_string}\n\nCell marked '@' is most likely to be empty with a chance of {:.2}% - No cells are definitively a bomb or safe\n", (probability * 100.0))
        }
    }

    pub fn from_string(input: String) -> Result<Board, String> {
        let mut board = Vec::new();
        let mut width = None;

        let lines = input.lines();

        for line_str in lines {
            if line_str.len() == 0 {
                continue;
            }
            if width.is_none() {
                width = Some(line_str.len());
            } else if width.unwrap() != line_str.len() {
                return Err(format!(
                    "Irregular line width - expected {} found {}",
                    width.unwrap(),
                    line_str.len()
                ));
            }

            let mut line = Vec::new();
            for c in line_str.chars() {
                line.push(CellTypes::from_char(c)?);
            }
            board.push(line);
        }

        if board.len() == 0 {
            return Err("Empty input".to_string());
        }

        let width = width.unwrap();
        let height = board.len();

        Ok(Board {
            board,
            width,
            height,
        })
    }

    pub fn complete_solvable(&mut self) -> bool {
        let mut change_made = false;

        for x in 0..self.width {
            for y in 0..self.height {
                let mut required = match &self.board[y][x] {
                    CellTypes::Value(v) => *v,
                    _ => continue,
                } as i32;
                if required == 0 { continue; }

                let mut possible_cells = Vec::new();

                for offset in [(-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1)] {
                    let (x, y) = (x as i32 + offset.0, y as i32 + offset.1);
                    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                        continue;
                    }

                    let (x, y) = (x as usize, y as usize);

                    match &self.board[y][x] {
                        CellTypes::Value(_) => continue,
                        CellTypes::Bomb => required -= 1,
                        CellTypes::Covered => possible_cells.push(offset),
                    };
                }

                if required == possible_cells.len() as i32 {
                    for offset in possible_cells {
                        let (x, y) = (x as i32 + offset.0, y as i32 + offset.1);
                        let (x, y) = (x as usize, y as usize);
                        self.board[y][x] = CellTypes::Bomb;
                        change_made = true;
                    }
                }
                else if required == 0 {
                    for offset in possible_cells {
                        let (x, y) = (x as i32 + offset.0, y as i32 + offset.1);
                        let (x, y) = (x as usize, y as usize);
                        self.board[y][x] = CellTypes::Value(0);
                        change_made = true
                    }
                }
            }
        }

        change_made
    }

    pub fn validate_board(&self) -> Result<usize, String> {
        let mut to_satisfy = 0;

        for x in 0..self.width {
            for y in 0..self.height {
                let mut required = match &self.board[y][x] {
                    CellTypes::Value(v) => *v,
                    _ => continue,
                } as i32;
                if required == 0 { continue; }

                let mut possible_cells: u8 = 0;
                for offset in [(-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1)] {
                    let (x, y) = (x as i32 + offset.0, y as i32 + offset.1);
                    if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                        continue;
                    }

                    let (x, y) = (x as usize, y as usize);

                    match &self.board[y][x] {
                        CellTypes::Value(_) => continue,
                        CellTypes::Bomb => required -= 1,
                        CellTypes::Covered => possible_cells += 1
                    };
                }

                if required < 0 {
                    return Err(format!("Cell at position [{}, {}] has {} bombs more than it should have", x, y, -required));
                }

                if (possible_cells as i32) < required {
                    return Err(format!("Cell at position [{}, {}] requires {} bomb(s) however only {} cell(s) can contain bombs", x, y, required, possible_cells));
                }

                if required != 0 {
                    to_satisfy += required as usize;
                }
            }
        }

        Ok(to_satisfy)
    }

    /// Solved, Open
    pub fn get_possible_boards(&self, visited: &mut HashSet<u64>, ignore: &mut HashSet<(usize, usize)>) -> (Vec<Board>, Vec<Board>) {
        let mut solved_boards = Vec::new();
        let mut open_boards = Vec::new();
        let current_satisfied = self.validate_board().unwrap();

        for x in 0..self.width {
            for y in 0..self.height {
                if ignore.contains(&(x, y)) {
                    continue
                }

                match &self.board[y][x] {
                    CellTypes::Covered => {},
                    _ => continue
                }

                let mut new_board = self.clone();
                new_board.board[y][x] = CellTypes::Bomb;

                let hash = new_board.get_hash();
                if visited.contains(&hash) {
                    continue;
                }
                visited.insert(hash);

                if let Ok(remaining) = new_board.validate_board() {
                    if remaining == 0 {
                        solved_boards.push(new_board);
                        continue;
                    }
                    else if remaining == current_satisfied {
                        ignore.insert((x, y));
                        continue;
                    }
                }
                else { continue; }
                open_boards.push(new_board);
            }
        }

        (solved_boards, open_boards)
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        for line in &self.board {
            for cell in line {
                cell.id().hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();

        for line in &self.board {
            for cell in line {
                output.push(cell.char());
            }
            output.push('\n');
        }

        output.remove(output.len() - 1);

        output
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}