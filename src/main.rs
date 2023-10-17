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

    //uci_protocol.handle_position(&"position fen 8/3P4/1p3b1p/p7/P7/1P3NPP/4p1K1/3k4 w - - 0 1 moves d7d8q".to_string());

    uci_protocol.start()
}
