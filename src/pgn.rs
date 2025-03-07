use std::fs;

use chess::{Board, ChessMove};

///The read_pgns function reads a file containing Portable Game Notation (PGN) formatted chess games,
///processes the notations, and returns a vector of move sequences in Long Algebraic Notation (LAN).
///
///The function ensures that only valid move sequences are retained
pub fn read_pgns(file_path: &str) -> Vec<Vec<String>> {
    let raw_pgns = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("should be able to read from: {}", file_path));

    let split_pgns = split_pgns(&raw_pgns);
    eprintln!("| got {} notations", split_pgns.len());

    let mut move_sequences: Vec<Vec<String>> = split_pgns
        .iter()
        .map(|notation| move_sequence(notation))
        .collect();

    validate(&mut move_sequences)
}

/// splits pgns across Vector
/// if provided &str contains only one pgn Vector's size is 1
fn split_pgns(pgns: &str) -> Vec<String> {
    let formatted_pgns = strip_metadata(pgns);
    let mut pgn_buff = Vec::new();
    let mut str_buff = String::new();

    for ln in formatted_pgns.iter() {
        str_buff.push_str(ln);

        if ln.contains("1-0") || ln.contains("0-1") || ln.contains("1/2") {
            pgn_buff.push(std::mem::take(&mut str_buff));
        }
    }

    pgn_buff
}

/// removes tags from pgn/s
fn strip_metadata(pgn: &str) -> Vec<String> {
    pgn.lines()
        .filter(|ln| !ln.is_empty())
        .filter(|ln| !ln.starts_with("["))
        .map(|ln| ln.to_string() + "\n")
        .collect()
}

/// split notation into individual moves
/// result aka 1-0 | 0-1 | 1/2-1/2 is not included
fn move_sequence(notation: &str) -> Vec<String> {
    let last_space = notation
        .rfind(' ')
        .expect("PGN is guaranteed to have space");

    notation[..last_space]
        .split_whitespace()
        .map(|mov| {
            // filter out | = | + | # |
            let cleaned: String = mov
                .chars()
                .filter(|&c| c != '+' && c != '#' && c != '=')
                .collect();

            match cleaned.find('.') {
                Some(dot) => cleaned[(dot + 1)..].to_string(),
                _ => cleaned,
            }
        })
        .collect()
}

/// validates pgn/s and transforms them to long algebraic notation from SAN
fn validate(move_sequences: &mut [Vec<String>]) -> Vec<Vec<String>> {
    eprintln!("| validating pgn/s");

    let mut buff = Vec::with_capacity(move_sequences.len());

    'outer: for (i, seq) in move_sequences.iter_mut().enumerate() {
        if seq.len() < 15 {
            eprintln!(
                "| ❌dropping notation {} as it's length is lower than 15",
                i + 1
            );
            continue;
        }

        let mut board = Board::default();
        for (j, mv) in seq.iter_mut().enumerate() {
            let possible_move = ChessMove::from_san(&board, mv);
            if possible_move.is_err() {
                eprintln!(
                    "| ❌{}-th move [{}] of {}th notation is not valid, removing notation",
                    (j + 1) / 2,
                    mv,
                    i + 1
                );

                continue 'outer;
            }

            let chess_move = possible_move.unwrap();
            *mv = chess_move.to_string();

            board = board.make_move_new(chess_move);
        }
        buff.push(seq.clone());
    }

    eprintln!(
        "|✅validated {} sequences out of {}",
        buff.len(),
        move_sequences.len()
    );

    buff
}
