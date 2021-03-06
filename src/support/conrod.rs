// Taken from https://github.com/PistonDevelopers/conrod/blob/master/backends/conrod_glium/examples/support/mod.rs

extern crate conrod_winit;

use crate::systems::{events as e, EventSystem};
use glium;
use std;
use std::time::{Duration, Instant};

pub struct GliumDisplayWinitWrapper(pub glium::Display);

impl conrod_winit::WinitWindow for GliumDisplayWinitWrapper {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.gl_window().get_inner_size().map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        self.0.gl_window().get_hidpi_factor() as _
    }
}

impl GliumDisplayWinitWrapper {
    pub fn update(&self, events: &EventSystem) {
        for event in events.events() {
            match event {
                e::AppEvent::Image(e::Image::Loaded { file, .. }) => {
                    let title = format!("{}", file);
                    log::info!("Set window title: {}", title);
                    self.0.gl_window().set_title(&title);
                }
                _ => (),
            }
        }
    }
}

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod_core::render::Primitives` to the
/// screen.
///
/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(
        &mut self,
        events_loop: &mut glium::glutin::EventsLoop,
        max_delay: Option<Duration>,
    ) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let default_frame_duration = Duration::from_millis(16);
        let frame_duration = match max_delay {
            None => default_frame_duration,
            Some(delay) => default_frame_duration.min(delay),
        };

        let duration_since_last_update = Instant::now().duration_since(last_update);
        if duration_since_last_update < frame_duration {
            std::thread::sleep(frame_duration - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        self.ui_needs_update = false;
        self.last_update = Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}
