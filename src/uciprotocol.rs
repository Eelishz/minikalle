// UCI Implementation from: https://wbec-ridderkerk.nl/html/UCIProtocol.html
// Engine also has some UCI output that is not handled through this module

use crate::search;
use core::panic;
use shakmaty::{fen::Fen, uci::Uci, Chess, Color, Move, Outcome, Position};
use std::io::stdin;

const LATENCY_MS: u64 = 100;

#[derive(Debug, PartialEq)]
enum Token {
    UCI,
    IsReady,
    SetOption,
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
    OptionName(String),
    OptionValue(String),
}

pub struct UciProtocol {
    chess_engine: search::Engine,
    position: Chess,
    n_moves: u16,
}

impl UciProtocol {
    pub fn new() -> UciProtocol {
        UciProtocol {
            chess_engine: search::Engine::new(),
            position: Chess::new(),
            n_moves: 0,
        }
    }

    pub fn demo(&mut self) {
        for _ in 0..50 {
            let (m, uci, _) = self
                .chess_engine
                .find_best_move(&self.position.clone(), 10_000, 6);
            self.position = self.position.clone().play(&m).unwrap();
            println!("bestmove {}", uci);
        }
    }

    fn new_game(&mut self) {
        self.n_moves = 0;
        self.chess_engine.new_game();
    }

    fn excecute_command(&mut self, tokens: &Vec<Token>) {
        for token in tokens {
            match token {
                Token::UCI => {
                    println!("id name minikalle");
                    println!("id author Eelis Holmstén");
                    println!("option name Hash type spin default 64 min 1 max 33554432");
                    println!("option name Book type check default true");
                    println!("option name NN type check default false");
                    println!("uciok");
                }
                Token::IsReady => println!("readyok"),
                Token::UciNewGame => self.new_game(),
                Token::FENStr(fenstr) => {
                    let fen: Fen = fenstr.parse().unwrap();
                    if let Ok(position) = fen.into_position(shakmaty::CastlingMode::Standard) {
                        self.position = position;
                    } else {
                        eprintln!("invalid fen");
                    }
                }
                Token::StartPos => {
                    let position = Chess::new();
                    self.position = position;
                }
                Token::Move(move_string) => {
                    let uci: Uci = move_string.parse().unwrap();

                    if let Ok(m) = uci.to_move(&self.position) {
                        self.position = self.position.clone().play(&m).unwrap();
                    } else {
                        eprintln!("UCI error");
                    }
                }
                Token::Go => self.handle_go(tokens),
                Token::Stop => self.stop_search(),
                Token::SetOption => self.set_option(tokens),
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

        let mut movetime: u64 = 0;

        let mut depth: u64 = 20;

        let mut prev_token = tokens.first().unwrap();
        for token in &tokens[1..] {
            match token {
                Token::Number(n) => match prev_token {
                    Token::WTime => wtime = *n,
                    Token::BTime => btime = *n,
                    Token::WInc => winc = *n,
                    Token::BInc => binc = *n,
                    Token::Depth => depth = *n,
                    Token::MoveTime => movetime = *n,
                    _ => movetime = 1000,
                },
                Token::Infinite => movetime = u64::MAX,
                _ => prev_token = token,
            }
        }

        let max_depth = depth as u8;
        let max_time = match turn {
            Color::White => wtime / 20 + winc,
            Color::Black => btime / 20 + binc,
        } - LATENCY_MS;

        let chess_move: Move;
        let uci: Uci;

        if movetime != 0 {
            (chess_move, uci, _) =
                self.chess_engine
                    .find_best_move(&self.position.clone(), movetime, max_depth);
        } else {
            (chess_move, uci, _) =
                self.chess_engine
                    .find_best_move(&self.position.clone(), max_time, max_depth);
        }

        self.n_moves += 1;
        if let Some(outcome) = self.position.outcome() {
            match outcome {
                Outcome::Draw => println!("info outcome 1/2-1/2"),
                Outcome::Decisive { winner } => match winner {
                    Color::White => println!("info outcome 1-0"),
                    Color::Black => println!("info outcome 0-1"),
                },
            }
        } else {
            self.position = self.position.clone().play(&chess_move).unwrap();
            println!("bestmove {}", uci);
        }
    }

    fn stop_search(&mut self) {}

    fn set_option(&mut self, tokens: &Vec<Token>) {
        // TODO: de-jank, add more options.
        // Should probably use a map or something.

        match tokens.get(1).unwrap() {
            Token::OptionName(x) => match x.as_str() {
                "Hash" => {
                    let value: usize = match tokens.last().unwrap() {
                        Token::OptionValue(x) => x.parse().unwrap(),
                        _ => panic!(),
                    };
                    self.chess_engine.set_hash(value);
                }
                "Book" => {
                    let value: bool = match tokens.last().unwrap() {
                        Token::OptionValue(x) => match x.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => true,
                        },
                        _ => true,
                    };
                    self.chess_engine.set_book(value);
                }
                "NN" => {
                    let value: bool = match tokens.last().unwrap() {
                        Token::OptionValue(x) => match x.as_str() {
                            "true" => true,
                            "false" => false,
                            _ => false,
                        },
                        _ => false,
                    };
                    self.chess_engine.set_nn(value);
                }
                _ => eprintln!("unkown option {x:?}"),
            },
            _ => eprintln!("parser error {tokens:?}"),
        }
    }

    fn parse_message(&mut self, message: &String) -> Vec<Token> {
        let mut split_message = message.split_whitespace();

        let mut tokens = vec![];
        let mut fen_buffer = String::new();
        let mut is_fen = false;

        loop {
            let Some(symbol) = split_message.next() else {
                break;
            };
            match symbol {
                "uci" => tokens.push(Token::UCI),
                "isready" => tokens.push(Token::IsReady),
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
                "setoption" => {
                    tokens.push(Token::SetOption);
                    assert_eq!(split_message.next().unwrap(), "name");
                    tokens.push(Token::OptionName(split_message.next().unwrap().to_string()));
                    assert_eq!(split_message.next().unwrap(), "value");
                    tokens.push(Token::OptionValue(
                        split_message.next().unwrap().to_string(),
                    ));
                }
                _ => {
                    if is_fen {
                        fen_buffer.push(' '); //trailing space to account for split method stripping whitespace
                        fen_buffer.push_str(symbol);

                        if self.is_fen_string(&fen_buffer[1..]) {
                            tokens.push(Token::FENStr(fen_buffer.clone()));
                            fen_buffer.clear();
                            is_fen = false;
                        }
                    } else if self.is_number(symbol) {
                        if let Ok(num) = symbol.parse::<u64>() {
                            tokens.push(Token::Number(num));
                        }
                    } else if self.is_move(symbol) {
                        tokens.push(Token::Move(symbol.to_string()));
                    }
                }
            }
        }

        if is_fen {
            eprintln!("unreconized fen string {message}");
        }

        tokens
    }

    fn is_fen_string(&mut self, symbol: &str) -> bool {
        symbol.parse::<Fen>().is_ok()
    }

    fn is_number(&mut self, symbol: &str) -> bool {
        symbol.parse::<u64>().is_ok()
    }

    fn is_move(&mut self, symbol: &str) -> bool {
        symbol.parse::<Uci>().is_ok()
    }

    pub fn put(&mut self, message: &String) {
        println!("{message}");
        let tokens = self.parse_message(message);
        self.excecute_command(&tokens);
    }

    pub fn start(&mut self) {
        eprintln!("minikalle by Eelis Holmstén");
        let mut message = String::new();

        while message != "quit" {
            message = String::new();
            stdin()
                .read_line(&mut message)
                .expect("Did not enter a correct string");
            message = message.trim().to_string();

            let tokens = self.parse_message(&message);
            self.excecute_command(&tokens);
        }
    }
}
