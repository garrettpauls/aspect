#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate conrod_derive;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate ttf_noto_sans;
extern crate log;
extern crate env_logger;

mod components;
mod data;
mod support;
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
        _ => ()
    }
}
