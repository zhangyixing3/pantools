use chrono::Local;
use env_logger::fmt::Target;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn init_logging() {
    //https://docs.rs/env_logger/0.10.0/env_logger/fmt/struct.Style.html
    Builder::new()
        .format(|buf, record| {
            let level = { buf.default_styled_level(record.level()) };
            let mut style = buf.style();
            style.set_bold(false);
            writeln!(
                buf,
                "{} [{}]{}:{:<4} {}",
                style.value(Local::now().format("%Y/%m/%d %H:%M")),
                level,
                record
                    .file()
                    .unwrap_or("None")
                    .split('/')
                    .last()
                    .unwrap_or("None"),
                record.line().unwrap_or(0),
                style.value(record.args())
            )
        })
        .target(Target::Stderr)
        .filter(None, LevelFilter::Debug)
        .init();
}
