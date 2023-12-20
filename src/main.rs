#![feature(test, slice_swap_unchecked)]

use std::env::args;

mod uciprotocol;

fn main() {
    let args = args();
    let mut uci_protocol = uciprotocol::UciProtocol::new();
    if args.last().unwrap() == "demo" {
        uci_protocol.demo();
    } else {
        uci_protocol.start();
    }
}
