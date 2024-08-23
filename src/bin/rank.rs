use rand::prelude::*;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::ChildStdin;
use std::process::{Child, Command, Stdio};
use std::thread;

struct Engine {
    handle: Child,
}

impl Engine {
    fn new(command: &[&str], uci_args: Option<&[&str]>) -> Self {
        let mut child = Command::new(command[0])
            .args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Could not spawn subprocess");

        if let Some(args) = uci_args {
            for arg in args {
                child.stdin.unwrap().write(arg.as_bytes()).unwrap();
                //stdin.write(&[b'\n']).unwrap();
                child.stdin.unwrap().flush().unwrap();
            }
            let mut buf = [0u8; 2048];
            &child.stdout.take().unwrap().read(&mut buf);
            println!("{}", String::from_utf8_lossy(&buf));
        }

        Engine { handle: child }
    }

    fn command(&mut self, cmd: String) {
        let mut stdin = self.handle.stdin.take().unwrap();
        stdin
            .write(cmd.as_bytes())
            .expect("failed to write to stdout");
    }
}

fn sleeps_ms(time: u64) {
    std::thread::sleep(std::time::Duration::from_millis(time));
}

fn get_random_position(fens: &Vec<String>) -> String {
    let i = thread_rng().gen_range(0..fens.len());
    fens[i].clone()
}

fn fens_from_file(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();

    text.lines().map(|e| e.to_string()).collect()
}

fn main() {
    let path = "random_positions";
    let fens = fens_from_file(path);

    for _ in 0..1 {
        let position = get_random_position(&fens);

        let mut engine_a = Engine::new(&["./stockfish-ubuntu-x86-64-avx2"], Some(&["uci"]));
        engine_a.command(format!("position fen {position}"));
        engine_a.command("go movetime 1000".to_string());
        let out = engine_a.handle.stdout.unwrap();
        for b in out.bytes() {
            print!("{}", b.unwrap() as char)
        }
    }
}
