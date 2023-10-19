use log::{error, info};
use shakmaty::{fen::Fen, uci::Uci, Chess, Color, Position};
use std::io::stdin;
mod engine;

pub struct UciProtocol {
    chess_engine: engine::Engine,
    position: Chess,
    playing_as: Color,
}

impl UciProtocol {
    pub fn new() -> UciProtocol {
        UciProtocol {
            chess_engine: engine::Engine::new(),
            position: Chess::new(),
            playing_as: Color::White,
        }
    }

    fn new_game(&mut self, _message: &String) {
        self.chess_engine.clear_repetition_table();
    }

    fn calc_think_time(&mut self, message: &String) -> u64 {
        match message
            .as_str()
            .split_whitespace()
            .nth(1)
            .expect("Think time error")
        {
            "mtime" | "movetime" => {
                return message
                    .split_whitespace()
                    .nth(2)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            }
            "wtime" => {
                if self.playing_as == Color::White {
                    return message
                        .split_whitespace()
                        .nth(2)
                        .unwrap()
                        .parse::<u64>()
                        .unwrap()
                        / 20;
                } else {
                    return message
                        .split_whitespace()
                        .nth(4)
                        .unwrap()
                        .parse::<u64>()
                        .unwrap()
                        / 20;
                }
            }
            _ => return 1500,
        }
    }

    fn handle_fen(&mut self, message: &String) {
        // dirty hack to allow 'moves' keyword after fen
        let mut i = 13;
        let mut moves: bool = false;
        for c in message[13..].chars() {
            i += 1;
            if c == 'm' {
                moves = true;
                break;
            }
        }

        let mut moves_index: usize = 0;
        for (i, cmd) in message.split_whitespace().enumerate() {
            if cmd == "moves" {
                moves_index = i + 1;
            }
        }

        let fen: Fen = if moves {
            message[13..i - 2].parse().expect("Fen failed")
        } else {
            message[13..].parse().expect("Fen failed")
        };

        let pos: Chess = fen
            .into_position(shakmaty::CastlingMode::Standard)
            .expect("Fen failed");
        self.position = pos;
        if moves {
            self.handle_moves(message, moves_index)
        }
    }

    fn handle_moves(&mut self, message: &String, skip_val: usize) {
        let moves = message.as_str().split_whitespace();

        for uci_str in moves.skip(skip_val) {
            let uci: Uci = uci_str.parse().expect("UCI parse failed");
            let chess_move = uci.to_move(&self.position).expect("UCI move failed");
            self.position = self.position.clone().play(&chess_move).unwrap();
        }
    }

    fn handle_startpos(&mut self, message: &String) {
        if message.as_str().split_whitespace().count() == 2 {
            info!("position reset");
            self.position = Chess::new();
            return;
        }
        match message
            .as_str()
            .split_whitespace()
            .nth(2)
            .expect("Splitting string failed")
        {
            "moves" => {
                self.position = Chess::new();
                self.handle_moves(message, 3);
            }
            _ => return,
        }
    }
    //position fen ...
    pub fn handle_position(&mut self, message: &String) {
        self.chess_engine.clear_repetition_table();
        match message
            .as_str()
            .split_whitespace()
            .nth(1)
            .expect("Splitting string failed")
        {
            "startpos" => self.handle_startpos(message),
            "fen" => self.handle_fen(message),
            _ => return,
        }
    }

    pub fn go(&mut self, message: &String) {
        self.playing_as = self.position.turn();
        let think_time = self.calc_think_time(message).clamp(0, 60_000);
        info!("thinking for {}ms", think_time);
        let (chess_move, uci, _) = self
            .chess_engine
            .find_best_move(self.position.clone(), think_time);
        self.position = self.position.clone().play(&chess_move).unwrap();
        println!("bestmove {}", uci);
    }

    pub fn start(&mut self) {
        let mut message = String::new();

        while message != "quit" {
            message = String::new();
            stdin()
                .read_line(&mut message)
                .expect("Did not enter a correct string");
            message = message.trim().to_string();

            info!("UCI message: {}", message);

            match message
                .as_str()
                .split_whitespace()
                .nth(0)
                .expect("Failed to split message")
            {
                "uci" => println!("uciok"),
                "isready" => println!("readyok"),
                "ucinewgame" => self.new_game(&message),
                "position" => self.handle_position(&message),
                "go" => self.go(&message),
                "stop" => continue,
                "quit" => break,
                "printboard" => println!("{:?}", self.position),
                _ => error!("unexpected UCI command {}", message),
            }
        }
    }
}
