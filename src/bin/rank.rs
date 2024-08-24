use rand::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(Debug)]
enum Outcome {
    White,
    Black,
    Draw,
}

struct Engine {
    pub handle: Child,
    output_buffer: Arc<Mutex<String>>,
}

impl Engine {
    fn new(command: &[&str], uci_args: Option<&[&str]>) -> Self {
        let mut child = Command::new(command[0])
            .args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Could not spawn subprocess");

        if let Some(args) = uci_args {
            if let Some(ref mut stdin) = child.stdin {
                for arg in args {
                    stdin.write(arg.as_bytes()).unwrap();
                    stdin.write(&[b'\n']).unwrap();
                }

                stdin.flush().unwrap();
            }
        }

        /*
        if let Some(ref mut stdout) = child.stdout.take() {
            let mut buf = [0u8; 1024 * 4];
            stdout.read_buf(&mut buf).unwrap();
            print!("{}", String::from_utf8_lossy(&buf));
        }
        */

        let output_buffer = Arc::new(Mutex::new(String::new()));
        if let Some(stdout) = child.stdout.take() {
            // Shared buffer to store output from the thread
            let output_buffer_clone = Arc::clone(&output_buffer);

            // Spawn a thread to read from stdout
            let _read_handle = thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                let mut local_buffer = String::new();

                loop {
                    // Read until EOF or timeout
                    match reader.read_line(&mut local_buffer) {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            // Append to the shared output buffer
                            let mut output = output_buffer_clone.lock().unwrap();
                            output.push_str(&local_buffer);
                            local_buffer.clear();
                        }
                        Err(err) => {
                            eprintln!("Failed to read from stdout: {}", err);
                            break;
                        }
                    }
                }
            });
        }

        Engine {
            handle: child,
            output_buffer,
        }
    }

    fn command(&mut self, cmd: String) {
        if let Some(ref mut stdin) = self.handle.stdin {
            stdin.write(cmd.as_bytes()).unwrap();
            stdin.write(&[b'\n']).unwrap();
            stdin.flush().unwrap();
        }
    }

    fn read_output(&self) -> String {
        self.output_buffer.lock().unwrap().clone()
    }

    fn read_last_line(&self) -> String {
        let output = self.read_output();
        output.split('\n').nth_back(1).unwrap().to_string()
    }

    fn read_uci_move(&self) -> String {
        self.read_last_line()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .to_string()
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

fn black_or_white(fen: &str) -> bool {
    match fen.split_whitespace().nth(1).unwrap() {
        "w" => true,
        "b" => false,
        _ => panic!(),
    }
}

fn simulate_game(engine_a: &mut Engine, engine_b: &mut Engine, position: String) -> Outcome {
    engine_a.command("ucinewgame".to_string());
    engine_b.command("ucinewgame".to_string());
    engine_a.command(format!("position fen {position}"));
    engine_b.command(format!("position fen {position}"));
    sleeps_ms(50);

    engine_a.command("go movetime 100".to_string());
    sleeps_ms(150);
    let mut moves = Vec::new();
    moves.push(engine_a.read_uci_move());

    let mut turn = false;
    let mut i = 0;
    loop {
        if i > 100 {
            return Outcome::Draw;
        }
        let engine = if turn { &mut *engine_a } else { &mut *engine_b };
        let command = format!("position fen {position} moves {}", moves.join(" "));
        engine.command(command);
        engine.command("go movetime 50".to_string());
        sleeps_ms(150);
        if engine.read_last_line().starts_with("info outcome") {
            break;
        }
        let uci_move = engine.read_uci_move();
        moves.push(uci_move);
        turn = !turn;
        i += 1;
    }

    let outcome = (if turn { engine_a } else { engine_b })
        .read_last_line()
        .split_whitespace()
        .last()
        .unwrap()
        .to_string();

    match outcome.as_str() {
        "1-0" => Outcome::White,
        "0-1" => Outcome::Black,
        "1/2-1/2" => Outcome::Draw,
        _ => panic!(),
    }
}

fn main() {
    let path = "random_positions";
    let fens = fens_from_file(path);
    let mut engine_a = Engine::new(&["./target/release/minikalle"], None);
    let mut engine_b = Engine::new(
        &["./target/release/minikalle"],
        Some(&["setoption name NN value true"]),
    );
    sleeps_ms(50);

    let mut a_score = 0;
    let mut b_score = 0;
    let mut draws = 0;

    let n = 1000;

    for i in 0..n {
        eprintln!("{i} / {n} - {a_score} {draws} {b_score}");
        let position = get_random_position(&fens);
        let a_starts = black_or_white(&position);
        let outcome = simulate_game(&mut engine_a, &mut engine_b, position);
        match outcome {
            Outcome::Draw => draws += 1,
            Outcome::White => {
                if a_starts {
                    a_score += 1
                } else {
                    b_score += 1
                }
            }
            Outcome::Black => {
                if a_starts {
                    b_score += 1
                } else {
                    a_score += 1
                }
            }
        }
        //eprintln!("{outcome:?}");
    }

    engine_a.command("quit".to_string());
    engine_b.command("quit".to_string());

    engine_a.handle.wait().unwrap();
    engine_b.handle.wait().unwrap();

    println!("{a_score} {draws} {b_score}")
}
