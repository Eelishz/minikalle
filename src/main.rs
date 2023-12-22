#![feature(test, slice_swap_unchecked)]

use std::env::args;

mod uciprotocol;
mod engine;
mod evaluation;
mod squaretables;
mod transpositiontable;
mod openings;

fn main() {
    let args = args();
    let mut uci_protocol = uciprotocol::UciProtocol::new();
    if args.last().unwrap() == "demo" {
        uci_protocol.demo();
    } else {
        uci_protocol.start();
    }
}
