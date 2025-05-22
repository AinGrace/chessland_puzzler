use std::io::BufRead;
use std::{
    fmt::{Debug, Display},
    io::{self, BufReader, BufWriter, Write as _},
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

use tracing::info;

pub fn best_move_for_pos_moves(moves: &str, depth: u8, stockfish: &mut Stockfish) -> String {
    stockfish.new_game().expect("can't start ucinewgame");

    let position_cmd = format!("position startpos moves {}", moves);
    let depth_cmd = format!("go depth {}", depth);

    stockfish
        .write(&position_cmd)
        .expect("can't write to stockfish");

    stockfish
        .write(&depth_cmd)
        .expect("can't write to stockfish");

    let output = stockfish.read_until("bestmove").unwrap();

    let best_move = output.split_whitespace().nth(1).unwrap();
    best_move.to_string()
}

pub fn eval_pos_moves(moves: &str, stockfish: &mut Stockfish) -> Evaluation {
    stockfish.new_game().expect("can't start ucinewgame");

    let position_cmd = format!("position startpos moves {moves}");
    let eval_cmd = "eval";

    stockfish
        .write(&position_cmd)
        .expect("could not write to stockfish");

    stockfish
        .write(eval_cmd)
        .expect("could not write to stockfish");

    let output = stockfish.read_until("Final").unwrap();

    if output.contains("in check") {
        return Evaluation::Check;
    }

    let eval_str = output.split_whitespace().nth(2).unwrap();
    let eval = eval_str
        .parse::<f32>()
        .unwrap_or_else(|err| panic!("could not parse {eval_str}: {err}"));

    Evaluation::Eval(eval)
}

/// Represents the evaluation of a chess position
pub enum Evaluation {
    /// Position where the side to move is in check
    Check,
    /// Numerical evaluation (positive favors white, negative favors black)
    Eval(f32),
}

impl Debug for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::Check => write!(f, "in check"),
            Evaluation::Eval(eval) => write!(f, "{eval}"),
        }
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluation::Check => write!(f, "in check"),
            Evaluation::Eval(eval) => write!(f, "{eval}"),
        }
    }
}

pub struct Stockfish {
    process: Child,
    writer: BufWriter<ChildStdin>,
    pub reader: BufReader<ChildStdout>,
}

impl Stockfish {
    pub fn try_init() -> Result<Self, io::Error> {
        let mut process = std::process::Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = process.stdin.take().expect("stockfish stdin error");
        let stdout = process.stdout.take().expect("stockfish stdout error");

        let writer = BufWriter::new(stdin);
        let reader = BufReader::new(stdout);

        Ok(Stockfish {
            process,
            writer,
            reader,
        })
    }

    /// Sends a command to the Stockfish engine
    ///
    /// # Arguments
    /// * `cmd` - The command string to send
    ///
    /// # Returns
    /// An io::Result indicating success or failure
    fn write(&mut self, cmd: &str) -> io::Result<()> {
        writeln!(self.writer, "{}", cmd)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resets the engine state for a new game
    ///
    /// # Returns
    /// An io::Result indicating success or failure
    fn new_game(&mut self) -> io::Result<()> {
        // Send new game command
        writeln!(self.writer, "ucinewgame")?;
        // Wait for engine to be ready
        writeln!(self.writer, "isready")?;
        self.writer.flush()?;
        self.read_until("readyok")?;
        Ok(())
    }

    /// Reads output from Stockfish until a specific marker is found
    fn read_until(&mut self, marker: &str) -> Result<String, io::Error> {
        let mut buffer = String::new();

        loop {
            buffer.clear();
            let bytes_read = self.reader.read_line(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            let trimmed = buffer.trim();
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Exit when marker is found
            if trimmed == marker || trimmed.contains(marker) {
                break;
            }
        }

        Ok(buffer)
    }
}

impl Drop for Stockfish {
    fn drop(&mut self) {
        let _ = self.write("quit");
        let _ = self.process.wait();
        info!("stockfish terminated successfully");
    }
}
