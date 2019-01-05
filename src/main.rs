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

use std::env::args;
use std::path::PathBuf;
use self::data::file_list::FileList;

fn run() {
    env_logger::init();

    let paths: Vec<_> = args().map(|a| PathBuf::from(a)).filter(|p| p.exists()).collect();
    let files = paths.iter().filter_map(|p| FileList::from_file(p)).next()
        .or(paths.iter().filter_map(|p| FileList::from_dir(p)).next());

    if let Some(files) = files {
        log::trace!("files: {:#?}", files);
        window::run();
    } else {
        log::error!("No image file or directory passed as an argument.");
    }
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
