use core::f32;
use shakmaty::{Color, EnPassantMode, Position, fen::Fen};
use std::{
    fmt::{Debug, Display, Write},
    str::FromStr,
};

use crate::stockfish::{self, Evaluation, Stockfish};
use shakmaty::{Chess, uci::UciMove};

/// Represents a chess puzzle with difficulty level, starting position, and solution moves
#[derive(Debug)]
pub struct Puzzle {
    pub lvl: PuzzleLevel,
    pub start_pos: String,
    pub notation: Vec<String>,
}

/// Defines puzzle difficulty levels
#[derive(Debug, Clone)]
pub enum PuzzleLevel {
    Easy,
    Medium,
    Hard,
}

/// Internal representation of a potential puzzle candidate
#[derive(Debug)]
struct PuzzleCandidate {
    original_pos: PositionData,
    delta: f32,
    side_to_move: Color,
}

/// Holds data about a specific chess position
struct PositionData {
    mv: String,
    fen: String,
    eval: Evaluation,
}

/// Display implementation for Puzzle - formats puzzle for output
impl Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Combine all moves into a space-separated string
        let moves = self.notation.iter().fold(String::new(), |mut acc, mv| {
            write!(acc, "{} ", mv).unwrap_or_else(|err| panic!("could not display puzzle: {err}"));
            acc
        });

        // Format as level|starting position|moves
        writeln!(f, "{}|{}|{}", self.lvl, self.start_pos, moves)
    }
}

impl PuzzleLevel {
    /// Converts difficulty level to a numeric value
    /// Used to determine how many moves to calculate for the puzzle
    fn as_number(&self) -> u8 {
        match self {
            Self::Easy => 1,
            Self::Medium => 2,
            Self::Hard => 3,
        }
    }
}

/// Display implementation for PuzzleLevel
impl Display for PuzzleLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PuzzleLevel::Easy => write!(f, "easy"),
            PuzzleLevel::Medium => write!(f, "medium"),
            PuzzleLevel::Hard => write!(f, "hard"),
        }
    }
}

/// Debug implementation for PositionData
impl Debug for PositionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n\tmv: {}\n\tfen: {}\n\teval: {}",
            self.mv, self.fen, self.eval
        )
    }
}

/// Generates a chess puzzle by analyzing a sequence of moves
///
/// # Arguments
/// * `lvl` - Difficulty level of the puzzle
/// * `moves` - Sequence of moves in UCI notation to analyze
/// * `stockfish` - Mutable reference to a Stockfish engine instance
///
/// # Returns
/// A Puzzle struct containing the generated puzzle
pub fn generate_puzzle_by_position_analysis(
    lvl: PuzzleLevel,
    moves: &[String],
    stockfish: &mut Stockfish,
) -> Puzzle {
    // Select a random range of moves to analyze
    let (from, to) = rand_range_of_moves(moves);

    // Prepare the chess board with the initial sequence of moves
    let mut board = prepare_board(&moves[0..from], Chess::default());
    let mut side_to_move = board.turn();
    let mut positions_and_eval = Vec::new();

    // Analyze each position in the selected range
    for i in from..to {
        // Analyze the actual move played
        let (pos_data, new_board) = analyze_pos(&moves[i], board, stockfish);
        board = new_board;

        // Find the best move in the position
        let best_pos_data = analyze_best_move(&pos_data.fen, board.clone(), stockfish);

        // Calculate the difference between played move and best move
        let delta = compute_delta(&pos_data.eval, &best_pos_data.eval);

        side_to_move = board.turn();

        let pos = PuzzleCandidate {
            original_pos: pos_data,
            delta,
            side_to_move,
        };

        positions_and_eval.push(pos);
    }

    // Find the position with the highest evaluation difference
    let hi_delta_pos_eval = highest_delta_position_for_side(positions_and_eval, side_to_move);

    // Prepare the sequence of moves for the puzzle
    let result_notation = prepare_result_notation(moves, hi_delta_pos_eval);

    // Finalize the puzzle by adding the appropriate number of solution moves
    finalize_puzzle(lvl, &result_notation, stockfish)
}

/// Finalizes the puzzle by adding the appropriate number of solution moves based on difficulty
///
/// # Arguments
/// * `lvl` - Difficulty level of the puzzle
/// * `moves` - Initial sequence of moves
/// * `stockfish` - Mutable reference to a Stockfish engine instance
///
/// # Returns
/// A complete Puzzle struct
fn finalize_puzzle(lvl: PuzzleLevel, moves: &[String], stockfish: &mut Stockfish) -> Puzzle {
    let lvl_num = lvl.as_number();
    let start_pos = moves.len() - 2;
    let mut notation = moves.to_vec();

    // Add additional solution moves based on puzzle difficulty
    // Easy: 2 moves, Medium: 4 moves, Hard: 6 moves
    for _ in 1..=(lvl_num * 2) {
        let mut board = Chess::default();
        board = prepare_board(&notation, board);

        // Get the current position in FEN notation
        let fen = Fen::from_position(board.clone(), EnPassantMode::Legal).to_string();

        // Use Stockfish to find the best move at depth 5
        let best_move = stockfish::best_move_for_pos(&fen, 5, stockfish);
        notation.push(best_move);
    }

    Puzzle {
        lvl,
        start_pos: moves[start_pos].clone(),
        notation,
    }
}

/// Prepares a chess board by applying a sequence of moves
///
/// # Arguments
/// * `moves` - Sequence of moves in UCI notation
/// * `board` - Initial chess board state
///
/// # Returns
/// The chess board after applying all moves
fn prepare_board(moves: &[String], mut board: Chess) -> Chess {
    for mv in moves.iter() {
        // Convert UCI string to UciMove object
        let uci = UciMove::from_str(mv).unwrap_or_else(|err| {
            eprintln!("{mv} is not valid uci move: {err}");
            eprintln!("moves -> {moves:?}");
            panic!()
        });

        // Convert UciMove to actual Move
        let mov = uci.to_move(&board).unwrap_or_else(|err| {
            eprintln!("cant convert {uci} to move: {err}");
            eprintln!("moves -> {moves:?}");
            panic!()
        });

        // Apply the move to the board
        board = board.play(&mov).unwrap_or_else(|err| {
            eprintln!("INVALID MOVE -> {err}");
            eprintln!("moves -> {moves:?}");
            panic!()
        });
    }

    board
}

/// Prepares the sequence of moves for the puzzle
///
/// # Arguments
/// * `moves` - Original sequence of moves
/// * `hi_delta_pos_eval` - The position with the highest evaluation difference
///
/// # Returns
/// A vector of moves leading up to the critical position
fn prepare_result_notation(moves: &[String], hi_delta_pos_eval: PuzzleCandidate) -> Vec<String> {
    let mut game = Chess::default();
    let mut fen_sequence = Vec::new();
    let mut result_notation = Vec::new();

    // Generate FEN for each position in the game
    for mv in moves.iter() {
        let uci = UciMove::from_str(mv).expect("always valid");
        let mov = uci.to_move(&game).expect("always valid");

        game = game.play(&mov).unwrap_or_else(|err| {
            eprintln!("INVALID MOVE -> {err}");
            panic!()
        });

        fen_sequence.push(Fen::from_position(game.clone(), EnPassantMode::Legal).to_string());
    }

    // Include moves up to the critical position
    for (i, fen) in fen_sequence.iter().enumerate() {
        if *fen != hi_delta_pos_eval.original_pos.fen {
            result_notation.push(moves[i].to_string());
        } else {
            result_notation.push(moves[i].to_string());
            break;
        }
    }

    result_notation
}

/// Finds the position with the highest evaluation difference for a given side
///
/// # Arguments
/// * `positions_and_eval` - Vector of position candidates with evaluations
/// * `side_to_move` - Which side to move (White or Black)
///
/// # Returns
/// The position with the highest evaluation difference
fn highest_delta_position_for_side(
    positions_and_eval: Vec<PuzzleCandidate>,
    side_to_move: Color,
) -> PuzzleCandidate {
    positions_and_eval
        .into_iter()
        .filter(|pos| pos.side_to_move == side_to_move)
        .max_by(|a, b| a.delta.total_cmp(&b.delta))
        .expect("never empty")
}

/// Analyzes a position after a move is played
///
/// # Arguments
/// * `mv` - Move in UCI notation
/// * `board` - Current chess board state
/// * `stockfish` - Mutable reference to a Stockfish engine instance
///
/// # Returns
/// A tuple containing position data and the new board state
fn analyze_pos(mv: &str, mut board: Chess, stockfish: &mut Stockfish) -> (PositionData, Chess) {
    let uci = UciMove::from_str(mv).expect("always valid");
    let mov = uci.to_move(&board).expect("always valid");

    // Apply the move to the board
    board = board.play(&mov).unwrap_or_else(|err| {
        eprintln!("INVALID MOVE -> {err}");
        panic!()
    });

    // Get the position in FEN notation
    let fen = Fen::from_position(board.clone(), EnPassantMode::Legal).to_string();

    // Evaluate the position with Stockfish
    let eval = stockfish::eval_pos(&fen, stockfish);

    let position = PositionData {
        mv: mv.to_string(),
        fen,
        eval,
    };

    (position, board)
}

/// Analyzes the best move in a position
///
/// # Arguments
/// * `fen` - FEN representation of the position
/// * `board` - Current chess board state
/// * `stockfish` - Mutable reference to a Stockfish engine instance
///
/// # Returns
/// Position data after the best move is played
fn analyze_best_move(fen: &str, mut board: Chess, stockfish: &mut Stockfish) -> PositionData {
    // Get the best move from Stockfish (depth 1)
    let best_move = stockfish::best_move_for_pos(fen, 1, stockfish);
    let uci = UciMove::from_str(&best_move).expect("always valid");
    let mov = uci.to_move(&board).expect("always valid");

    // Apply the best move to the board
    board = board.play(&mov).unwrap_or_else(|err| {
        eprintln!("INVALID MOVE -> {err}");
        panic!()
    });

    // Get the new position in FEN notation
    let fen = Fen::from_position(board.clone(), EnPassantMode::Legal).to_string();

    // Evaluate the new position
    let eval = stockfish::eval_pos(&fen, stockfish);

    PositionData {
        mv: best_move,
        fen,
        eval,
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
fn rand_range_of_moves(moves: &[String]) -> (usize, usize) {
    // Start from one-third of the way through the moves
    let from: usize = moves.len() / 3;

    // End at a random point between start+1 and the end
    let to: usize = rand::random_range(from + 1..moves.len() - 1);

    (from, to)
}
