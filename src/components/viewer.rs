use conrod_core::{widget, Positionable, Sizeable, Widget};

use crate::systems::{events as e, EventSystem};

widget_ids!(struct Ids {
    image,
});

pub enum ImageScale {
    FitAll,
    Scale {
        scale: f64,
        offset_top: f64,
        offset_left: f64,
    },
}

pub struct ImageData {
    id: conrod_core::image::Id,
    w: u32,
    h: u32,
}

pub struct State {
    ids: Ids,
    image: Option<ImageData>,
    scale: ImageScale,
}

#[derive(WidgetCommon)]
pub struct ImageViewer<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    events: &'a EventSystem,
}

impl<'a> ImageViewer<'a> {
    pub fn new(events: &'a EventSystem) -> Self {
        ImageViewer {
            common: widget::CommonBuilder::default(),
            events,
        }
    }
}

impl<'a> Widget for ImageViewer<'a> {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            image: None,
            scale: ImageScale::FitAll,
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, id, .. } = args;

        for event in self.events.events() {
            match event {
                e::AppEvent::Image(event) => match event {
                    e::Image::SwapImageId(id) => {
                        log::info!("Swapping image id to {:?}", id);
                        state.update(|s| {
                            if let Some(i) = s.image.as_mut() {
                                i.id = *id
                            }
                        });
                    }
                    e::Image::Loaded { id, w, h, .. } => {
                        log::info!("Loading new image: {:?}, {}x{}", id, w, h);
                        state.update(|s| {
                            s.image = Some(ImageData {
                                id: *id,
                                w: *w,
                                h: *h,
                            });
                            s.scale = ImageScale::FitAll;
                        });
                    }
                },
                _ => (),
            }
        }

        if let Some(image) = &state.image {
            let [uw, uh] = ui.wh_of(id).unwrap_or(ui.window_dim());
            let scaled = ScaledImage::new(&image, &state.scale, uw, uh);

            widget::Image::new(image.id)
                .parent(id)
                .graphics_for(id)
                .w_h(scaled.w, scaled.h)
                .top_left_with_margins(scaled.top, scaled.left)
                .set(state.ids.image, ui);

            let input = ui.widget_input(id);
            for drag in input.drags() {
                use conrod_core::input::MouseButton;
                if let Some(scale) = match (drag.button, &state.scale) {
                    (
                        MouseButton::Left,
                        ImageScale::Scale {
                            scale,
                            offset_top,
                            offset_left,
                        },
                    ) => Some(ImageScale::Scale {
                        scale: *scale,
                        offset_top: offset_top - drag.delta_xy[1],
                        offset_left: offset_left + drag.delta_xy[0],
                    }),
                    (MouseButton::Left, _) => Some(ImageScale::Scale {
                        scale: scaled.scale,
                        offset_top: scaled.top,
                        offset_left: scaled.left,
                    }),
                    _ => None,
                } {
                    state.update(|s| s.scale = scale);
                }
            }

            for scroll in input.scrolls() {
                let scale = match state.scale {
                    ImageScale::FitAll => ImageScale::Scale {
                        scale: scaled.scale,
                        offset_top: scaled.top,
                        offset_left: scaled.left,
                    },
                    ImageScale::Scale {
                        scale,
                        offset_top,
                        offset_left,
                    } => ImageScale::Scale {
                        scale: adjust_scale(scale, &scroll),
                        offset_top,
                        offset_left,
                    },
                };
                state.update(|s| s.scale = scale);
            }
        }
    }
}

fn adjust_scale(scale: f64, scroll: &conrod_core::event::Scroll) -> f64 {
    use conrod_core::input::ModifierKey;
    const MIN_SCALE: f64 = 0.01;
    const MAX_SCALE: f64 = 100.0;

    let factor = match scroll.modifiers {
        ModifierKey::CTRL => 1000.0,
        ModifierKey::SHIFT => 250.0,
        _ => 500.0,
    };

    (scale + (scroll.y / -factor)).max(MIN_SCALE).min(MAX_SCALE)
}

struct ScaledImage {
    scale: f64,
    w: f64,
    h: f64,
    top: f64,
    left: f64,
}

impl ScaledImage {
    fn new(
        image: &ImageData,
        scale: &ImageScale,
        full_width: f64,
        full_height: f64,
    ) -> ScaledImage {
        let w = image.w as f64;
        let h = image.h as f64;

        match scale {
            ImageScale::FitAll => {
                let scale = (full_width / w).min(full_height / h);
                let w = scale * w;
                let h = scale * h;

                ScaledImage {
                    scale,
                    w,
                    h,
                    left: (full_width - w) / 2.0,
                    top: (full_height - h) / 2.0,
                }
            }
            ImageScale::Scale {
                scale,
                offset_top,
                offset_left,
            } => {
                let w = scale * w;
                let h = scale * h;
                ScaledImage {
                    scale: *scale,
                    w,
                    h,
                    left: *offset_left,
                    top: *offset_top,
                }
            }
        }
    }
}
