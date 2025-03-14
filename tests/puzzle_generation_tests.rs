use shakmaty::{FromSetup, Position, fen::Fen};
use std::str::FromStr;

use chessland_puzzle_generator::{
    pgn,
    puzzle::{PuzzleLevel, generate_puzzle_by_position_analysis},
    stockfish::Stockfish,
};
use rand::{Rng, rng};
use shakmaty::{Chess, uci::UciMove};

#[test]
fn evaluated_puzzle() {
    let notations = pgn::read_pgns("Ding.pgn");
    let mut stockfish = Stockfish::default();

    for i in 1..5 {
        eprintln!("GENERATING {i}th PUZZLE");
        let rand_notation = rng().random_range(0..notations.len());
        eprintln!("pgn num {rand_notation}");
        let pos =
            generate_puzzle_by_position_analysis(PuzzleLevel::Hard, &notations[rand_notation], &mut stockfish);

        let mut board = Chess::default();

        for mv in pos.notation.iter() {
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

        eprintln!("puzzle -> {pos}");
    }
}

#[test]
fn test_correctness() {
    let fen: Fen = "6k1/p2b2pp/1p6/2pPQq2/2P2p2/5N2/PP4PP/6K1 w - - 1 30"
        .parse()
        .unwrap();
    
    let mov = "e5b8";
    let uci_mov = UciMove::from_str(&mov).unwrap();

    let game = Chess::from_setup(fen.into_setup(), shakmaty::CastlingMode::Standard).unwrap();
    let game_move = uci_mov.to_move(&game).unwrap();

    game.play(&game_move).unwrap();
}
