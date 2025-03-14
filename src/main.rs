use chessland_puzzle_generator::{
    pgn,
    puzzle::{self, PuzzleLevel, generate_puzzle_by_position_analysis},
    stockfish::Stockfish,
};
use rand::{Rng, rng};
use shakmaty::{Chess, Position, uci::UciMove};
use std::{env, fs, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("provide at least one argument");
        std::process::exit(1);
    }

    // split pgn/s across vector
    let notations = pgn::read_pgns(&args[1]);

    let mut stockfish = Stockfish::default();

    // buffer for puzzles
    let mut puzzles = String::new();

    // generate easy puzzles
    for _ in 0..5 {
        let random = rng().random_range(0..notations.len());
        let puzzle = generate_puzzle_by_position_analysis(
            PuzzleLevel::Easy,
            &notations[random],
            &mut stockfish,
        );

        check_correctness(&puzzle);
        puzzles.push_str(&puzzle.to_string());
    }

    // generate medium puzzles
    for _ in 0..5 {
        let random = rng().random_range(0..notations.len());
        let puzzle = generate_puzzle_by_position_analysis(
            PuzzleLevel::Medium,
            &notations[random],
            &mut stockfish,
        );

        check_correctness(&puzzle);
        puzzles.push_str(&puzzle.to_string());
    }

    // generate hard puzzles
    for _ in 0..5 {
        let random = rng().random_range(0..notations.len());
        let puzzle = generate_puzzle_by_position_analysis(
            PuzzleLevel::Hard,
            &notations[random],
            &mut stockfish,
        );

        check_correctness(&puzzle);
        puzzles.push_str(&puzzle.to_string());
    }

    eprintln!("generated {} puzzles", puzzles.lines().count());
    fs::write("Puzzles.txt", puzzles).unwrap();
}

fn check_correctness(puzzle: &puzzle::Puzzle) {
    let mut board = Chess::default();

    for mv in puzzle.notation.iter() {
        let uci = UciMove::from_str(mv).unwrap_or_else(|err| {
            eprintln!("{mv} is not valid uci move: {err}");
            panic!()
        });

        let mov = uci.to_move(&board).unwrap_or_else(|err| {
            eprintln!("cant convert {uci} to move: {err}");
            panic!()
        });
        board = board.play(&mov).unwrap_or_else(|err| {
            eprintln!("INVALID MOVE -> {err}");
            panic!()
        });
    }
}
