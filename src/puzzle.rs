use std::{
    fmt::{Display, Write},
    io::Read,
    str::FromStr,
};

use chess::{Board, ChessMove};
use rand::Rng;

use crate::stockfish;

#[derive(Debug)]
pub struct Puzzle {
    lvl: PuzzleLevel,
    start_pos: String,
    notation: Vec<String>,
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let moves = self.notation.iter().fold(String::new(), |mut acc, mv| {
            write!(acc, "{} ", mv).unwrap_or_else(|err| panic!("could not display puzzle: {err}"));
            acc
        });
        writeln!(f, "{}|{}|{}", self.lvl, self.start_pos, moves)
    }
}

#[derive(Debug, Clone)]
pub enum PuzzleLevel {
    Easy = 1,
    Medium = 2,
    Hard = 3,
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

pub fn generate_puzzle(lvl: PuzzleLevel, moves: &[String], board: Board) -> Puzzle {
    let mut move_set = get_move_set(moves);
    let start_pos = move_set.last().expect("should not be empty").clone();
    let lvl_num = lvl.clone() as u8;

    for _ in 1..=lvl_num {
        let cmd = generate_command(&move_set, board);
        let best_move = get_best_move(cmd);
        move_set.push(best_move);
    }

    Puzzle {
        lvl,
        start_pos,
        notation: move_set,
    }
}

fn get_move_set(moves: &[String]) -> Vec<String> {
    let from = moves.len() / 3;
    let to = from * 2;
    let rand_move = rand::rng().random_range(from..to);

    moves[0..=rand_move].to_vec()
}

fn generate_command(moves: &[String], mut board: Board) -> (String, String) {
    for i in 0..moves.len() {
        let chess_move = ChessMove::from_str(&moves[i]).unwrap_or_else(|_| {
            panic!(
                "should be a valid move MOVE -> {}| INDEX -> {} | MOVES -> {moves:#?}",
                &moves[i], i
            )
        });
        board = board.make_move_new(chess_move);
    }

    let depth = 1;
    let fen = format!("{board}");

    let position_cmd = format!("position fen {fen}");
    let go_cmd = format!("go depth {depth}");

    (position_cmd, go_cmd)
}

fn get_best_move(cmd: (String, String)) -> String {
    let mut buffer = String::new();
    let mut stockfish = stockfish::StockfishBuilder::default()
        .write(&cmd.0)
        .write(&cmd.1)
        .build()
        .unwrap_or_else(|err| {
            panic!("can't build read-only stockfish: {err}");
        });

    stockfish
        .reader
        .read_to_string(&mut buffer)
        .unwrap_or_else(|err| panic!("can't write to buffer: {err}"));

    let best_move: String = buffer
        .lines()
        .filter(|ln| ln.contains("bestmove"))
        .map(|ln| ln.split_whitespace().nth(1).unwrap())
        .collect();

    best_move
}
