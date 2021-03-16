extern crate base64;
extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate scrap;

mod capture;
mod ui;
mod utils;

use std::env;

fn main() -> iced::Result {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();
    info!("main: booting");
    ui::run()
}
