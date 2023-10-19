#![feature(slice_partition_dedup)]

mod uciprotocol;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("log/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    let mut uci_protocol = uciprotocol::UciProtocol::new();

    //for _ in 1..100 {
    //    uci_protocol.go(&"go mtime 10000".to_string());
    //}

    uci_protocol.start()
}
