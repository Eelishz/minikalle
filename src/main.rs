#![feature(test)]

mod uciprotocol;

fn main() {
    let mut uci_protocol = uciprotocol::UciProtocol::new();
    uci_protocol.start()
}
