#[macro_use]
extern crate conrod_core;
#[macro_use]
extern crate conrod_derive;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate ttf_noto_sans;

mod components;
mod support;
mod window;

fn main() {
    match std::panic::catch_unwind(|| window::run()) {
        Err(e) => {
            // TODO: show error to user properly
            eprintln!("{:?}", e);
        }
        _ => ()
    }
}
