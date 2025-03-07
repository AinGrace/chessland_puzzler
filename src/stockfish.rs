use std::{
    error::Error,
    io::{Read, Write},
    process::{Child, ChildStdin, ChildStdout, Stdio},
};

pub struct Stockfish {
    pub process: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}

impl Stockfish {
    pub fn kill(mut self) -> Result<(), Box<dyn Error>> {
        self.process.kill()?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<bool, Box<dyn Error>> {
        self.stdin.write_all("ucinewgame\n".as_bytes())?;
        self.stdin.write_all("isready\n".as_bytes())?;

        let mut buff = [0; 100];
        self.stdout.read(&mut buff)?;

        for _ in 0..10 {
            if std::str::from_utf8(&buff).unwrap().contains("readyok") {
                break;
            }

            buff.fill(0);
            self.stdout.read(&mut buff).unwrap();
        }

        let response = std::str::from_utf8(&buff)?;

        if response.contains("readyok") {
            Ok(true)
        } else {
            Ok(false)
        }
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

        Stockfish {
            process: stockfish,
            stdin,
            stdout,
        }
    }
}
