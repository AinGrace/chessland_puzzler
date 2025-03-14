use shakmaty::{Chess, Position, san::San};
use std::{fs, str::FromStr};

/// Reads chess games from a PGN file and converts them to sequences of UCI moves
///
/// # Arguments
/// * `file_path` - Path to the PGN file to read
///
/// # Returns
/// * Vector of validated chess move sequences in UCI format
///
/// # Panics
/// * If the file cannot be read
pub fn read_pgns(file_path: &str) -> Vec<Vec<String>> {
    // Read file contents or panic if file cannot be read
    let raw_pgns = fs::read_to_string(file_path)
        .unwrap_or_else(|err| panic!("should be able to read from {file_path}: {err}"));

    // Split the contents into individual PGN notations
    let split_pgns = split_pgns(&raw_pgns);
    eprintln!("| got {} notations", split_pgns.len());

    // Extract move sequences from each notation
    let move_sequences: Vec<Vec<String>> = split_pgns
        .iter()
        .map(|notation| move_sequence(notation))
        .collect();

    // Validate the move sequences and return only valid ones
    validate(move_sequences)
}

/// Splits a string containing multiple PGN notations into separate games
///
/// # Arguments
/// * `pgns` - String containing one or more PGN notations
///
/// # Returns
/// * Vector of individual PGN notation strings
fn split_pgns(pgns: &str) -> Vec<String> {
    // Remove metadata and empty lines
    let formatted_pgns = strip_metadata(pgns);

    let mut notation = Vec::new();
    let mut line = String::new();

    // Combine lines and split at game result markers
    for ln in formatted_pgns.iter() {
        line.push_str(ln);
        // Check for game ending markers
        if ln.contains("1-0") || ln.contains("0-1") || ln.contains("1/2") {
            notation.push(std::mem::take(&mut line));
        }
    }

    notation
}

/// Removes metadata lines and empty lines from PGN content
///
/// # Arguments
/// * `pgn` - Raw PGN content
///
/// # Returns
/// * Vector of lines without metadata and empty lines
fn strip_metadata(pgn: &str) -> Vec<String> {
    pgn.lines()
        .filter(|ln| !ln.is_empty()) // Remove empty lines
        .filter(|ln| !ln.starts_with("[")) // Remove metadata lines (start with [)
        .map(|ln| ln.to_string() + "\n") // Add newline to each line
        .collect()
}

/// Extracts the move sequence from a PGN notation string
///
/// # Arguments
/// * `notation` - A PGN notation string
///
/// # Returns
/// * Vector of chess moves in SAN format
///
/// # Panics
/// * If the notation doesn't contain any spaces
fn move_sequence(notation: &str) -> Vec<String> {
    // Find the last space in the notation (before the game result)
    let last_space = notation
        .rfind(' ')
        .expect("PGN is guaranteed to have space");

    // Split the notation into moves and remove move numbers
    notation[..last_space]
        .split_whitespace()
        .map(|mv| match mv.find('.') {
            Some(dot) => mv[(dot + 1)..].to_string(), // Remove move number (e.g., "1." from "1.e4")
            None => mv.to_string(),                   // Keep the move as is if no dot found
        })
        .collect()
}

/// Validates move sequences and converts them to UCI format
///
/// # Arguments
/// * `move_sequences` - Vector of move sequences in SAN format
///
/// # Returns
/// * Vector of valid move sequences converted to UCI format
fn validate(mut move_sequences: Vec<Vec<String>>) -> Vec<Vec<String>> {
    eprintln!("| validating pgn/s");

    move_sequences.retain_mut(|seq| {
        // Remove games with fewer than 15 moves
        if seq.len() / 2 < 15 {
            eprintln!("| ❌dropping notation as its length is lower than 15");
            return false;
        }

        // Initialize a chess board to track position
        let mut board = Chess::default();

        // Validate each move and convert to UCI format
        for mv in seq.iter_mut() {
            match San::from_str(mv) {
                Ok(san) => match san.to_move(&board) {
                    Ok(chess_move) => {
                        // Convert to UCI format and update the move
                        *mv = chess_move
                            .to_uci(shakmaty::CastlingMode::Standard)
                            .to_string();
                        // Update the board position
                        board = board.play(&chess_move).expect("always valid");
                    }
                    Err(err) => {
                        eprintln!("invalid san {san}: {err}");
                        return false;
                    }
                },
                Err(err) => {
                    eprintln!("| ❌move [{mv}] is invalid, removing notation: {err}");
                    return false;
                }
            }
        }
        true
    });

    eprintln!("|✅validated {} sequences", move_sequences.len());
    move_sequences
}
