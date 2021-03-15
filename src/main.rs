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
    // Set log level to make sure
    if env::var("RUST_LOG").unwrap_or("debug".into()) != "info" {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    info!("main: booting");
    ui::run()
}
