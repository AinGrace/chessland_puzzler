use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

use chess::{Board, ChessMove};
use rand::Rng;

use crate::stockfish::Stockfish;

#[derive(Debug)]
pub struct Puzzle {
    pub lvl: u8,
    pub start_pos: String,
    pub notation: Vec<String>,
}

pub enum PuzzleLevel {
    Easy,
    Medium,
    Hard,
}

impl PuzzleLevel {
    fn numeric(&self) -> u8 {
        match self {
            PuzzleLevel::Easy => 1,
            PuzzleLevel::Medium => 2,
            PuzzleLevel::Hard => 3,
        }
    }
}

impl Display for PuzzleLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PuzzleLevel::Easy => write!(f, "easy"),
            PuzzleLevel::Medium => write!(f, "medium"),
            PuzzleLevel::Hard => write!(f, "hard"),
        }
    }
}

pub fn generate_puzzle(
    lvl: PuzzleLevel,
    moves: &Vec<String>,
    board: Board,
    stockfish: &mut Stockfish,
) -> Puzzle {
    let mut slice = get_slice(moves.to_vec());
    let start_pos = slice.last().expect("should not be empty").clone();

    for _ in 1..=lvl.numeric() {
        let cmd = generate_command(&slice, board);
        let best_move = get_best_move(cmd, stockfish);
        slice.push(best_move);
    }

    Puzzle {
        lvl: lvl.numeric(),
        start_pos,
        notation: slice,
    }
}

fn get_slice(mut moves: Vec<String>) -> Vec<String> {
    let from = moves.len() / 3;
    let to = from * 2;
    let rand_move = rand::rng().random_range(from..to);

    moves.drain(0..=rand_move).collect()
}

fn generate_command(ref moves: &[String], mut board: Board) -> (String, String) {
    for i in 0..moves.len() {
        let chess_move = ChessMove::from_str(&moves[i]).expect(&format!(
            "should be a valid move MOVE -> {}| INDEX -> {} | MOVES -> {moves:#?}",
            &moves[i], i
        ));
        board = board.make_move_new(chess_move);
    }

    let depth = 1;
    let fen = format!("{board}");

    let position_cmd = format!("position fen {fen}\n");
    let go_cmd = format!("go depth {depth}\n");

    (position_cmd, go_cmd)
}

fn get_best_move(cmd: (String, String), stockfish: &mut Stockfish) -> String {
    let mut buffer = [0; 100];

    if !stockfish.reset().unwrap() {
        panic!("can't reset stockfish");
    }

    stockfish.stdin.write(cmd.0.as_bytes()).unwrap();
    stockfish.stdin.write(cmd.1.as_bytes()).unwrap();

    loop {
        if std::str::from_utf8(&buffer).unwrap().contains("best") {
            break;
        }

        buffer.fill(0);
        stockfish.stdout.read(&mut buffer).unwrap();
    }

    let best_move = std::str::from_utf8(&buffer)
        .unwrap()
        .lines()
        .filter_map(|ln| {
            if ln.contains("bestmove") {
                return Some(ln.split_whitespace().nth(1).unwrap());
            }
            None
        })
        .collect();

    best_move
}
