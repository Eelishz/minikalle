#![feature(test, slice_swap_unchecked, const_trait_impl)]

use std::env::args;

mod benchmark;
mod engine;
mod evaluation;
mod neural_eval;
mod openings;
mod squaretables;
mod transpositiontable;
mod uciprotocol;

fn main() {
    let args = args();
    let mut uci = uciprotocol::UciProtocol::new();
    let mode = args.last().unwrap();
    match mode.as_str() {
        "demo" => uci.demo(),
        "benchmark" => benchmark::benchmark(),
        _ => uci.start(),
    };
}
