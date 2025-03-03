use std::io::{Read, Write};

use chess::{Board, ChessMove};
use rand::Rng;

use crate::stockfish::Stockfish;

pub fn lvl1_puzzle(moves: &Vec<String>, mut board: Board, stockfish: &mut Stockfish) -> Puzzle {
    println!("INFO: generating lvl 1 puzzle");
    let len = moves.len();
    let mid = len / 2;
    let rand_move = rand::rng().random_range(mid..len - 1);

    for i in 0..=rand_move {
        let chess_move = ChessMove::from_san(&board, &moves[i]).expect(&format!(
            "should be a valid move MOVE -> {}| INDEX -> {} | MOVES -> {moves:#?}",
            &moves[i], i
        ));
        board = board.make_move_new(chess_move);
    }

    let depth = 1;
    let fen = format!("{board}");

    let position_cmd = format!("position fen {fen}\n");
    let go_cmd = format!("go depth {depth}\n");

    let mut buffer = [0; 50];

    stockfish.stdin.write(position_cmd.as_bytes()).unwrap();
    stockfish.stdin.write(go_cmd.as_bytes()).unwrap();

    loop {
        if std::str::from_utf8(&buffer).unwrap().contains("best") {
            break;
        }

        buffer.fill(0);
        stockfish.stdout.read(&mut buffer).unwrap();
    }

    let binding = std::str::from_utf8(&buffer)
        .unwrap()
        .lines()
        .filter(|ln| ln.contains("bestmove"))
        .collect::<String>();
    let best_move = binding.splitn(3, " ").collect::<Vec<_>>()[1];

    println!("best move: {best_move}");
    stockfish.kill().expect("could not kill stockfish");

    let mut notation = moves[0..=rand_move].to_vec();
    notation.push(best_move.to_string());

    Puzzle {
        lvl: 1,
        start_pos: moves[rand_move].clone(),
        notation,
    }
}

fn lvl2_puzzle<'a>() -> Puzzle {
    todo!()
}

fn lvl3_puzzle<'a>() -> Puzzle {
    todo!()
}

#[derive(Debug)]
pub struct Puzzle {
    pub lvl: u8,
    pub start_pos: String,
    pub notation: Vec<String>,
}

