extern crate directories;
extern crate iced;
extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate scrap;

mod capture;
mod ui;
mod utils;

use std::env;
use std::fs;
use std::path::Path;

fn main() -> iced::Result {
    // Set sane defaults
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");
    pretty_env_logger::init();
    // Search for config
    let dirs = if let Some(dirs) = directories::ProjectDirs::from("app", "kyoyu", "kyoyu") {
        dirs
    } else {
        panic!("kyoyu: config: couldn't get config path");
    };

    let config_path = {
        let path = dirs.config_dir();
        fs::create_dir_all(path).expect("kyoyu: config: couldn't recursively create directory");
        path.join(Path::new("config.toml"))
    };
    info!("kyoyu: config: eventually load from: {}", config_path.to_str().unwrap());

    info!("main: booting");
    ui::run()
}
