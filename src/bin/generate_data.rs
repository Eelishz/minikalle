use crate::pgn::PgnIterator;
use shakmaty::{san::San, Chess, Position};
use std::{env, io::stdin};

mod pgn {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufReader, Read};

    #[derive(Debug)]
    pub struct Game {
        pub moves: Vec<String>,
        pub headers: HashMap<String, String>,
    }

    pub struct PgnIterator {
        pgn: BufReader<File>,
    }

    impl PgnIterator {
        pub fn new(path: String) -> Self {
            let file = File::open(path).expect("cannot open pgn file");
            let pgn = BufReader::new(file);

            PgnIterator { pgn }
        }
    }

    impl Iterator for PgnIterator {
        type Item = Game;

        fn next(&mut self) -> Option<Self::Item> {
            // This is a shit praser but it is faster than python.

            let mut game = Game {
                moves: vec![],
                headers: HashMap::new(),
            };

            let mut is_game = false;
            let mut read_key = true;
            let mut prev_newline = 0;
            let mut key = String::new();
            let mut value = String::new();
            let mut move_buf = String::new();

            let mut buf = [0; 1];
            let mut i = 0;

            while self.pgn.read(&mut buf).is_ok() {
                let c = buf[0] as char;

                // header parsing
                if !is_game {
                    match c {
                        '[' => read_key = true,
                        ' ' => read_key = false,
                        ']' => {
                            game.headers.insert(key.clone(), value.clone());
                            key.clear();
                            value.clear();
                        }
                        '\n' => {
                            if i - 1 == prev_newline {
                                is_game = true;
                            }
                            prev_newline = i;
                        }
                        _ => {
                            if read_key {
                                key.push(c)
                            } else {
                                value.push(c)
                            }
                        }
                    }
                } else {
                    match c {
                        '?' => (),
                        '!' => (),
                        ' ' => {
                            game.moves.push(move_buf.clone());
                            move_buf.clear();
                        }
                        '\n' => {
                            if i - 1 == prev_newline {
                                return Some(game);
                            }
                            prev_newline = i;
                        }
                        _ => move_buf.push(c),
                    }
                }
                i += 1;
            }

            return None;
        }
    }
}

fn serialize(position: &Chess) -> [i32; 768] {
    let board = position.board();

    let mut result = [0; 768];
    let mut index = 0;

    let white = board.white();
    let black = board.black();

    let p = board.pawns();
    let n = board.knights();
    let b = board.bishops();
    let r = board.rooks();
    let q = board.queens();
    let k = board.kings();

    for color in [white, black] {
        for piece in [p, n, b, r, q, k] {
            let bb = color.intersect(piece);

            for sq in bb {
                let sq = sq as usize;
                result[index + sq] = 1;
            }
            index += 64;
        }
    }

    return result;
}

fn main() {
    let mut args = env::args();
    let _program = args.next();
    let pgn = PgnIterator::new(args.next().unwrap());
    let flags = args.next();
    let interactive = if let Some(flags) = flags {
        flags == "--interactive"
    } else {
        false
    };
    // let pgn = PgnIterator::new("lichess_db_standard_rated_2013-01.pgn".to_string());

    let n_max = 250_000_000;
    let mut n = 0;

    let mut buf = String::new();

    'outer: for game in pgn {
        let mut board = Chess::default();
        let moves = game.moves;
        let headers = game.headers;

        let termination = headers.get("Termination").unwrap();

        if termination != r#""Normal""# {
            continue;
        }

        let result = headers.get("Result").unwrap();
        let y = match result.as_str() {
            r#""1-0""# => 1,
            r#""0-1""# => -1,
            r#""1/2-1/2""# => 0,
            x => panic!("unknown result {}", x),
        };

        for move_string in moves {
            let san: San = match move_string.parse() {
                Ok(m) => m,
                _ => continue,
            };
            let m = san.to_move(&board).unwrap();
            board = board.play(&m).unwrap();

            if board.capture_moves().len() != 0 {
                continue;
            }

            let ser = serialize(&board);

            buf.push_str(&y.to_string());
            for x in ser {
                buf.push(',');
                buf.push_str(&x.to_string());
            }
            buf.push('\n');
            if interactive && buf.len() > 0 {
                print!("{buf}");
                buf.clear();
                let mut stdin_buffer = String::new();
                while stdin_buffer.trim() != "next" {
                    stdin_buffer.clear();
                    let stdin = stdin();
                    stdin.read_line(&mut stdin_buffer).unwrap();
                }
            }
            n += 1;

            if n >= n_max {
                break 'outer;
            }
        }

        if !interactive {
            print!("{buf}");
            buf.clear();
        }
    }
    if !interactive {
        print!("{buf}");
        buf.clear();
    }
}
