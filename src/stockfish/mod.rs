use std::{
    error::Error,
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

pub struct Stockfish {
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}

impl Stockfish {
    pub fn kill(&mut self) -> Result<(), Box<dyn Error>> {
        self.process.kill()?;
        Ok(())
    }

    pub fn new() -> Stockfish {
        let mut stockfish = std::process::Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("stockfish failed to start");

        let stdin = stockfish.stdin.take().expect("stockfish stdin error");
        let stdout = stockfish.stdout.take().expect("stockfish stdout error");

        Stockfish {
            process: stockfish,
            stdin,
            stdout,
        }
    }
}
