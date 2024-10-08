#![feature(test, slice_swap_unchecked, const_trait_impl, portable_simd)]

use std::env::args;

mod benchmark;
mod evaluation;
mod neural_eval;
mod openings;
mod search;
mod transpositiontable;
mod uciprotocol;

fn main() {
    let mut args = args();
    let mut uci = uciprotocol::UciProtocol::new();
    let mode = args.nth(1).unwrap_or("".to_string());
    match mode.as_str() {
        "demo" => uci.demo(),
        "benchmark" => benchmark::benchmark(args.last().unwrap_or("".to_string()).to_string()),
        _ => uci.start(),
    };
}
