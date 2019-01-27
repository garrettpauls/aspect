extern crate chrono;
#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate conrod_derive;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate env_logger;
extern crate gif;
extern crate gif_dispose;
extern crate image;
extern crate log;
extern crate rand;
extern crate ttf_noto_sans;

mod components;
mod data;
mod res;
mod support;
mod systems;
mod theme;
mod window;

fn run() {
    env_logger::init();
    window::run();
}

fn main() {
    match std::panic::catch_unwind(&run) {
        Err(e) => {
            // TODO: show error to user properly
            eprintln!("{:?}", e);
        }
        _ => (),
    }
}
