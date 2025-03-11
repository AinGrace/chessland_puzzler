use std::{
    error::Error,
    fmt::Write as _,
    io::Write as _,
    io::{self, BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

struct Stockfish {
    process: Child,
    writer: BufWriter<ChildStdin>,
    reader: BufReader<ChildStdout>,
}

impl Stockfish {
    fn drop_write(self) -> ReadStockfish {
        drop(self.writer);

        ReadStockfish {
            process: self.process,
            reader: self.reader,
        }
    }

    fn write(&mut self, position: &str) -> io::Result<()> {
        writeln!(self.writer, "{}", position)
    }
}

impl Default for Stockfish {
    fn default() -> Self {
        let mut stockfish = std::process::Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("stockfish failed to start");

        let stdin = stockfish.stdin.take().expect("stockfish stdin error");
        let stdout = stockfish.stdout.take().expect("stockfish stdout error");

        let writer = BufWriter::new(stdin);
        let reader = BufReader::new(stdout);

        Stockfish {
            process: stockfish,
            writer,
            reader,
        }
    }
}

pub struct ReadStockfish {
    process: Child,
    pub reader: BufReader<ChildStdout>,
}

impl Drop for ReadStockfish {
    fn drop(&mut self) {
        self.process.kill().unwrap_or_else(|err| {
            panic!("could not properly kill stockfish: {err}");
        });
    }
}

pub struct StockfishBuilder {
    errors: Vec<Box<dyn Error>>,
    commands: String,
}

impl StockfishBuilder {
    pub fn new() -> StockfishBuilder {
        StockfishBuilder {
            errors: Vec::new(),
            commands: String::new(),
        }
    }

    pub fn write(mut self, command: &str) -> Self {
        let result = writeln!(self.commands, "{}", command);
        if result.is_err() {
            self.errors.push(Box::new(result.unwrap_err()));
        }

        self
    }

    pub fn build(self) -> Result<ReadStockfish, Box<dyn Error>> {
        let mut stockfish = Stockfish::default();

        for cmd in self.commands.lines() {
            stockfish.write(cmd)?;
        }

        let read_stockfish = stockfish.drop_write();

        Ok(read_stockfish)
    }
}
