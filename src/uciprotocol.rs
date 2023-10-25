// UCI Implementation from: https://wbec-ridderkerk.nl/html/UCIProtocol.html
// Engine also has some UCI output that is not handled through this module

use log::{error, info, warn};
use regex::Regex;
use shakmaty::{fen::Fen, uci::Uci, Chess, Color, Move, Position};
use std::io::stdin;
mod engine;

#[derive(Debug, PartialEq)]
enum Token {
    UCI,
    IsReady,
    SetOption, //unimplemented
    Register,
    UciNewGame,
    Position,
    FEN,
    StartPos,
    Moves,
    Go,
    SearchMoves,
    Ponder,
    WTime,
    BTime,
    WInc,
    BInc,
    MovesToGo,
    Depth,
    Nodes,
    Mate,
    MoveTime,
    Infinite,
    Stop,
    PonderHit, //unimplemented
    Quit,
    Move(String),
    Number(u64),
    FENStr(String),
}

pub struct UciProtocol {
    chess_engine: engine::Engine,
    position: Chess,
}

impl UciProtocol {
    pub fn new() -> UciProtocol {
        UciProtocol {
            chess_engine: engine::Engine::new(),
            position: Chess::new(),
        }
    }

    fn new_game(&mut self) {
        self.chess_engine.clear_repetition_table();
    }

    fn excecute_fen(&mut self, tokens: &Vec<Token>) {
        for token in tokens {
            match token {
                Token::UCI => println!("uciok"),
                Token::IsReady => println!("readyok"),
                Token::UciNewGame => self.new_game(),
                Token::FENStr(fenstr) => {
                    let fen: Fen = fenstr.parse().unwrap();
                    let position: Chess =
                        fen.into_position(shakmaty::CastlingMode::Standard).unwrap();
                    self.position = position;
                }
                Token::StartPos => {
                    let position = Chess::new();
                    self.position = position;
                }
                Token::Move(move_string) => {
                    let uci: Uci = move_string.parse().unwrap();
                    let m = uci.to_move(&self.position).unwrap();
                    self.position = self.position.clone().play(&m).unwrap();
                }
                Token::Go => self.handle_go(tokens),
                Token::Stop => self.stop_search(),
                _ => (),
            }
        }
    }

    fn handle_go(&mut self, tokens: &Vec<Token>) {
        let turn = self.position.turn();

        let mut wtime: u64 = 1000;
        let mut btime: u64 = 1000;
        let mut winc: u64 = 1000;
        let mut binc: u64 = 1000;

        let movetime: u64 = 0;
        let mut infinite = false;

        let mut depth: u64 = 40;

        let mut prev_token = tokens.first().unwrap();
        for token in &tokens[1..] {
            match token {
                Token::Number(n) => match prev_token {
                    Token::WTime => wtime = *n,
                    Token::BTime => btime = *n,
                    Token::WInc => winc = *n,
                    Token::BInc => binc = *n,
                    Token::Depth => depth = *n,
                    _ => (),
                },
                Token::Infinite => infinite = true,
                _ => prev_token = &token,
            }
        }

        let max_time: u64;
        let max_depth = depth as u8;

        match turn {
            Color::White => max_time = (wtime / 20 + winc) as u64,
            Color::Black => max_time = (btime / 20 + binc) as u64,
        }

        let chess_move: Move;
        let uci: Uci;

        if infinite {
            (chess_move, uci, _) =
                self.chess_engine
                    .find_best_move(self.position.clone(), 999999999999, max_depth);
        } else if movetime != 0 {
            (chess_move, uci, _) =
                self.chess_engine
                    .find_best_move(self.position.clone(), movetime, max_depth);
        } else {
            (chess_move, uci, _) =
                self.chess_engine
                    .find_best_move(self.position.clone(), max_time, max_depth);
        }

        self.position = self.position.clone().play(&chess_move).unwrap();

        println!("bestmove {}", uci);
    }

    fn stop_search(&mut self) {}

    fn parse_message(&mut self, message: &String) {
        let split_message = message.split_whitespace();

        let mut tokens = vec![];
        let mut fen_buffer = String::new();
        let mut is_fen = false;

        for symbol in split_message {
            match symbol {
                "uci" => tokens.push(Token::UCI),
                "isready" => tokens.push(Token::IsReady),
                "setoption" => tokens.push(Token::SetOption),
                "register" => tokens.push(Token::Register),
                "ucinewgame" => tokens.push(Token::UciNewGame),
                "position" => tokens.push(Token::Position),
                "fen" => {
                    tokens.push(Token::FEN);
                    is_fen = true;
                    fen_buffer.clear();
                }
                "startpos" => tokens.push(Token::StartPos),
                "moves" => tokens.push(Token::Moves),
                "go" => tokens.push(Token::Go),
                "searchmoves" => tokens.push(Token::SearchMoves),
                "ponder" => tokens.push(Token::Ponder),
                "wtime" => tokens.push(Token::WTime),
                "btime" => tokens.push(Token::BTime),
                "winc" => tokens.push(Token::WInc),
                "binc" => tokens.push(Token::BInc),
                "movestogo" => tokens.push(Token::MovesToGo),
                "depth" => tokens.push(Token::Depth),
                "nodes" => tokens.push(Token::Nodes),
                "mate" => tokens.push(Token::Mate),
                "movetime" => tokens.push(Token::MoveTime),
                "infinite" => tokens.push(Token::Infinite),
                "stop" => tokens.push(Token::Stop),
                "ponderhit" => tokens.push(Token::PonderHit),
                "quit" => tokens.push(Token::Quit),
                _ => {
                    if is_fen {
                        fen_buffer.push(' '); //trailing space to account for split method stripping whitespace
                        fen_buffer.push_str(symbol);

                        if self.is_fen_string(&fen_buffer[1..]) {
                            println!("match!");
                            tokens.push(Token::FENStr(fen_buffer.clone()));
                            fen_buffer.clear();
                            is_fen = false;
                        }
                    } else {
                        if self.is_number(symbol) {
                            if let Ok(num) = symbol.parse::<u64>() {
                                tokens.push(Token::Number(num));
                            }
                        } else if self.is_move(symbol) {
                            tokens.push(Token::Move(symbol.to_string()));
                        }
                    }
                }
            }
        }

        if is_fen {
            eprintln!("unreconized fen strigng {message}");
        }

        self.excecute_fen(&tokens);
    }

    fn is_fen_string(&mut self, symbol: &str) -> bool {
        // https://gist.github.com/Dani4kor/e1e8b439115878f8c6dcf127a4ed5d3e
        let re_fen = Regex::new(r"\s*^(((?:[rnbqkpRNBQKP1-8]+\/){7})[rnbqkpRNBQKP1-8]+)\s([b|w])\s([K|Q|k|q]{1,4})\s(-|[a-h][1-8])\s(\d+\s\d+)$").unwrap();
        re_fen.is_match(symbol)
    }

    fn is_number(&mut self, symbol: &str) -> bool {
        symbol.parse::<u64>().is_ok()
    }

    fn is_move(&mut self, symbol: &str) -> bool {
        let re_move = Regex::new(r"^[a-h][1-8][a-h][1-8][qrbn]?$").unwrap();
        re_move.is_match(symbol)
    }

    pub fn start(&mut self) {
        println!("minikalle by Eelis Holmstén");
        let mut message = String::new();

        while message != "quit" {
            message = String::new();
            stdin()
                .read_line(&mut message)
                .expect("Did not enter a correct string");
            message = message.trim().to_string();

            info!("UCI message: {}", message);

            self.parse_message(&message);
        }
    }
}
