use core::f32;
use std::ops::RangeInclusive;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::stockfish;
use crate::domain::stockfish::{Evaluation, Stockfish};

use super::pgn::{InvalidNotationError, Pgn};

/// Represents a chess puzzle with position, and solution moves
#[derive(Debug, Serialize, Deserialize)]
pub struct Puzzle {
    pub moves: Vec<Move>,
    #[serde(rename = "startPositionOfPuzzle")]
    pub start_pos: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Move {
    from: String,
    to: String,
    promotion: Option<String>,
}

pub struct InvalidMoveFormat;

impl FromStr for Move {
    type Err = InvalidMoveFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 4 {
            let (from, to) = s.split_at(2);
            return Ok(Move {
                from: from.to_string(),
                to: to.to_string(),
                promotion: None,
            });
        }
        if s.len() == 5 {
            let (from, to_and_prom) = s.split_at(2);
            let (to, prom) = to_and_prom.split_at(2);
            return Ok(Move {
                from: from.to_string(),
                to: to.to_string(),
                promotion: Some(prom.to_string()),
            });
        }

        Err(InvalidMoveFormat)
    }
}

/// Holds data about a specific chess position
struct PositionData {
    pos: usize,
    best_mv: String,
    delta: f32,
}

// impl Display for Puzzle {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // Combine all moves into a space-separated string
//         let moves = self.notation.iter().fold(String::new(), |mut acc, mv| {
//             write!(acc, "{} ", mv).unwrap_or_else(|err| panic!("could not display puzzle: {err}"));
//             acc
//         });

//         writeln!(f, "{}|{}", self.start_pos, moves)
//     }
// }

/// Generates a chess puzzle by analyzing a sequence of moves
///
/// # Arguments
/// * `pgn` - Sequence of moves in UCI notation to analyze
/// * `stockfish` - Mutable reference to a Stockfish engine instance
///
/// # Returns
/// A Puzzle struct containing the generated puzzle
pub fn generate_puzzle_by_position_analysis(
    moves: &str,
    stockfish: &mut Stockfish,
) -> Result<Puzzle, InvalidNotationError> {
    let pgn = Pgn::from_str(moves)?;

    let best_position = rand_range_of_moves(&pgn)
        .map(|move_idx| analyze_pos(move_idx, &pgn, stockfish))
        .max_by(|x, y| x.delta.total_cmp(&y.delta))
        .expect("always valid");

    let mut puzzle_moves: Vec<String> = pgn
        .moves()
        .iter()
        .take(best_position.pos)
        .map(|a| a.to_string())
        .collect();

    puzzle_moves.push(best_position.best_mv);
    
    let final_moves: Result<Vec<Move>, InvalidMoveFormat> = puzzle_moves.iter().map(|mov| Move::from_str(mov)).collect();
    match final_moves {
        Ok(moves) => {
            Ok(Puzzle {
                start_pos: best_position.pos,
                moves,
            })
        },
        Err(_) => Err(InvalidNotationError("unexpected error on final stage of move generation".to_string())),
    }
}

fn analyze_pos(last_move: usize, moves: &Pgn, stockfish: &mut Stockfish) -> PositionData {
    let base_moves = moves
        .moves()
        .iter()
        .take(last_move)
        .cloned()
        .collect::<Pgn>()
        .to_string();

    let eval = stockfish::eval_pos_moves(&base_moves, stockfish);

    let best_mv = stockfish::best_move_for_pos_moves(&base_moves, 5, stockfish);
    let full_moves = format!("{base_moves} {best_mv}");

    let best_eval = stockfish::eval_pos_moves(&full_moves, stockfish);
    let delta = compute_delta(&eval, &best_eval);

    PositionData {
        pos: last_move,
        best_mv,
        delta,
    }
}

/// Computes the absolute difference between two position evaluations
///
/// # Arguments
/// * `pos_eval` - Evaluation of the current position
/// * `best_move_eval` - Evaluation after the best move
///
/// # Returns
/// The absolute difference between evaluations
fn compute_delta(pos_eval: &Evaluation, best_move_eval: &Evaluation) -> f32 {
    match (pos_eval, best_move_eval) {
        // If both are numerical evaluations, return absolute difference
        (Evaluation::Eval(pos_val), Evaluation::Eval(best_val)) => (pos_val - best_val).abs(),

        // If one is in check, use the absolute value of the other
        (_, Evaluation::Eval(best_val)) => best_val.abs(),

        // If both are in check, return infinity
        (_, _) => f32::INFINITY,
    }
}

/// Generates a random range of moves to analyze
///
/// # Arguments
/// * `moves` - Total sequence of moves
///
/// # Returns
/// A tuple containing the start and end indices of the range
fn rand_range_of_moves(moves: &Pgn) -> RangeInclusive<usize> {
    // Start from one-third of the way through the moves
    let from: usize = moves.moves().len() / 3;

    // End at a random point between start+1 and the end
    let to: usize = rand::random_range(from + 1..moves.moves().len() - 1);

    from..=to
}
