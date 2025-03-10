use std::{
    io::{self, BufReader, BufWriter, Write},
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

pub struct Stockfish {
    process: Child,
    writer: BufWriter<ChildStdin>,
    pub reader: BufReader<ChildStdout>,
}

pub struct RStockfish {
    process: Child,
    pub reader: BufReader<ChildStdout>,
}

// TODO: implement builder pattern
impl Stockfish {
    pub fn drop_write(self) -> RStockfish {
        drop(self.writer);

        RStockfish {
            process: self.process,
            reader: self.reader,
        }
    }

    pub fn write(&mut self, position: &str) -> io::Result<()> {
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

impl Drop for RStockfish {
    fn drop(&mut self) {
        self.process.kill().unwrap_or_else(|err| {
            panic!("could not properly kill stockfish: {err}");
        });
    }
}
