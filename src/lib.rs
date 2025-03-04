mod pgn;
mod puzzle;
mod stockfish;

use chess::Board;
use stockfish::Stockfish;

pub fn run() {
    let notations = pgn::read_pgns("Berliner.pgn");

    let puzzle = puzzle::lvl1_puzzle(&notations[0], Board::default(), &mut Stockfish::new());

    eprintln!("Puzzle -> {puzzle:#?}")
}
