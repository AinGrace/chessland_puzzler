use std::io::BufRead;
use std::{
    fmt::{Debug, Display},
    io::{self, BufReader, BufWriter, Write as _},
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

/// Determines the best chess move for a given position
///
/// # Arguments
/// * `fen` - A string slice containing the chess position in FEN notation
/// * `depth` - The search depth for the Stockfish engine
/// * `stockfish` - A mutable reference to a Stockfish instance
///
/// # Returns
/// A String containing the best move in UCI notation (e.g. "e2e4")
pub fn best_move_for_pos(fen: &str, depth: u8, stockfish: &mut Stockfish) -> String {
    // Reset engine state for a new game
    stockfish.new_game().expect("can't start ucinewgame");

    // Prepare commands to set position and search depth
    let position_cmd = format!("position fen {}", fen);
    let depth_cmd = format!("go depth {}", depth);

    // Send position to engine
    stockfish
        .write(&position_cmd)
        .expect("can't write to stockfish");

    // Start the search with specified depth
    stockfish
        .write(&depth_cmd)
        .expect("can't write to stockfish");

    // Read output until "bestmove" is found
    let output = stockfish.read_until("bestmove").unwrap();

    // Extract and return the best move
    let best_move = output.split_whitespace().nth(1).unwrap();
    best_move.to_string()
}

/// Evaluates a chess position
///
/// # Arguments
/// * `fen` - A string slice containing the chess position in FEN notation
/// * `stockfish` - A mutable reference to a Stockfish instance
///
/// # Returns
/// An Evaluation enum with either a numeric evaluation or indication of check
pub fn eval_pos(fen: &str, stockfish: &mut Stockfish) -> Evaluation {
    // Reset engine state for a new game
    stockfish.new_game().expect("can't start ucinewgame");

    // Prepare and send position command
    let position_cmd = format!("position fen {fen}");
    let eval_cmd = "eval";

    stockfish
        .write(&position_cmd)
        .expect("could not write to stockfish");

    // Request evaluation
    stockfish
        .write(eval_cmd)
        .expect("could not write to stockfish");

    // Read output until "Final" evaluation is found
    let output = stockfish.read_until("Final").unwrap();

    // Special case: if position is in check
    if output.contains("in check") {
        return Evaluation::Check;
    }

    // Parse the numerical evaluation
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
    ///
    /// # Arguments
    /// * `marker` - The string to look for in the output
    ///
    /// # Returns
    /// Result containing either the output string or an IO error
    fn read_until(&mut self, marker: &str) -> Result<String, io::Error> {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            let bytes_read = self.reader.read_line(&mut buffer)?;

            // Exit if no more output
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

/// Default implementation creates a new Stockfish process
impl Default for Stockfish {
    fn default() -> Self {
        // Start the Stockfish process
        let mut process = std::process::Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("stockfish failed to start");

        // Get stdin/stdout handles
        let stdin = process.stdin.take().expect("stockfish stdin error");
        let stdout = process.stdout.take().expect("stockfish stdout error");

        // Create buffered reader and writer
        let writer = BufWriter::new(stdin);
        let reader = BufReader::new(stdout);

        Stockfish {
            process,
            writer,
            reader,
        }
    }
}

/// Ensures Stockfish process is properly terminated when the struct is dropped
impl Drop for Stockfish {
    fn drop(&mut self) {
        // Send quit command
        let _ = self.write("quit");
        // Wait for process to terminate
        let _ = self.process.wait();
        eprintln!("stockfish terminated successfully");
    }
}
