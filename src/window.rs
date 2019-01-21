use conrod_core::Widget;
use glium::Surface;

use ttf_noto_sans;

use crate::components::App;
use crate::support::{EventLoop, GliumDisplayWinitWrapper, LogError};
use crate::systems::{self, events as e};
use crate::res::Resources;

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 500;

widget_ids!(struct Ids {
    app,
});

pub fn run() {
    let mut event_system = systems::EventSystem::new();

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
    let mut image_system = systems::ImageSystem::new(&display.0);
    let mut file_list = crate::data::FileList::from_environment();
    if let Some(file_list) = &file_list {
        if let Some(file) = file_list.current() {
            event_system.push(e::AppEvent::Load(file.clone()));
        }
    }

    let resources = Resources::load(&mut image_system).unwrap();

    let ids = Ids::new(ui.widget_id_generator());

    let mut event_loop = EventLoop::new();
    'main: loop {
        event_system.update();

        for event in event_loop.next(&mut events_loop, image_system.time_to_next_update()) {
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

        image_system.update(&mut event_system).log_err();
        if let Some(files) = &mut file_list {
            files.update(&mut event_system);
        }

        {
            use conrod_core::{Positionable, Sizeable};
            let ui = &mut ui.set_widgets();
            App::new(&mut event_system, &resources, &file_list)
                .parent(ui.window)
                .wh_of(ui.window)
                .top_left()
                .set(ids.app, ui);
        }

        if let Some(primitives) = ui.draw_if_changed() {
            let image_map = image_system.get_map();
            renderer.fill(&display.0, primitives, image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
