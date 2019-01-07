use conrod_core::Widget;
use glium::Surface;

use ttf_noto_sans;

use crate::components::{Action, App, ImageManager};
use crate::support::{EventLoop, GliumDisplayWinitWrapper};

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 500;

widget_ids!(struct Ids {
    app,
});

pub fn run() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Aspect")
        .with_dimensions((INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = GliumDisplayWinitWrapper(display);

    let mut ui = conrod_core::UiBuilder::new([INITIAL_WINDOW_WIDTH as f64, INITIAL_WINDOW_HEIGHT as f64]).build();
    ui.fonts.insert(conrod_core::text::FontCollection::from_bytes(ttf_noto_sans::REGULAR).unwrap().into_font().unwrap());
    ui.theme = super::theme::default_theme();

    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();
    let mut image_manager = ImageManager::new();

    let ids = Ids::new(ui.widget_id_generator());

    let mut event_loop = EventLoop::new();
    'main: loop {
        for event in event_loop.next(&mut events_loop) {
            if let Some(event) = conrod_winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::CloseRequested => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        {
            use conrod_core::{Positionable, Sizeable};
            let ui = &mut ui.set_widgets();
            for action in App::new(image_manager.current().clone())
                .parent(ui.window)
                .wh_of(ui.window)
                .top_left()
                .set(ids.app, ui) {
                match action {
                    Action::LoadImage(path) => if let Err(e) = image_manager.load_image(&display.0, &path) { log::error!("Failed to load image {}: {}", path.display(), e) },
                    _ => ()
                }
            }
        }

        if let Some(primitives) = ui.draw_if_changed() {
            let image_map = image_manager.get_map();
            renderer.fill(&display.0, primitives, image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
