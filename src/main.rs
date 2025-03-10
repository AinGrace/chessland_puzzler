use std::fs;

use chess::Board;
use chessland_puzzle_generator::{
    pgn,
    puzzle::{PuzzleLevel, generate_puzzle},
};
use rand::Rng;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("provide at least one argument");
        std::process::exit(1);
    }

    eprintln!("parsing notations");
    let notations = pgn::read_pgns(&args[1]); // split .pgn file across Vector
    let mut puzzles = Vec::with_capacity(10); // temp value, for now generating only 10 puzzles

    eprintln!("generating easy puzzles");
    // easy puzzles
    for _ in 0..5 {
        let rand_notation = rand::rng().random_range(0..notations.len());
        puzzles.push(generate_puzzle(
            PuzzleLevel::Easy,
            &notations[rand_notation],
            Board::default(),
        ));
    }

    eprintln!("generating medium puzzles");
    // medium puzzles
    for _ in 0..3 {
        let rand_notation = rand::rng().random_range(0..notations.len());
        puzzles.push(generate_puzzle(
            PuzzleLevel::Medium,
            &notations[rand_notation],
            Board::default(),
        ));
    }

    eprintln!("generating hard puzzles");
    // hard puzzles
    for _ in 0..2 {
        let rand_notation = rand::rng().random_range(0..notations.len());
        puzzles.push(generate_puzzle(
            PuzzleLevel::Hard,
            &notations[rand_notation],
            Board::default(),
        ));
    }

    eprintln!("generated {} puzzles", puzzles.len());

    let mut buffer = String::new();

    for puzzl in puzzles {
        buffer.push_str(&puzzl.to_string());
    }

    fs::write("puzzles.txt",  buffer).expect("should write to a file")
}
