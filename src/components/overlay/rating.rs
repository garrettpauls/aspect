use conrod_core::{widget, widget::Id, Widget, Sizeable, Positionable};

use crate::data::Rating;
use crate::res::Resources;

struct Ids {
    background: Id,
    stars: [Id; 5],
}

impl Ids {
    fn new(mut generator: widget::id::Generator) -> Self {
        Ids {
            background: generator.next(),
            stars: [generator.next(), generator.next(), generator.next(), generator.next(), generator.next()],
        }
    }
}


pub struct State {
    ids: Ids,
}

#[derive(WidgetCommon)]
pub struct StarRating<'a> {
    #[conrod(common_builder)] common: widget::CommonBuilder,
    rating: Option<Rating>,
    res: &'a Resources,
}

impl<'a> StarRating<'a> {
    pub fn new(rating: Option<Rating>, res: &'a Resources) -> Self {
        StarRating {
            common: widget::CommonBuilder::default(),
            rating,
            res,
        }
    }
}

impl<'a> Widget for StarRating<'a> {
    type State = State;
    type Style = ();
    type Event = Option<Option<Rating>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {}

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            state,
            ui,
            id,
            rect,
            ..
        } = args;

        widget::Rectangle::fill_with(rect.dim(), ui.theme.shape_color)
            .parent(id).graphics_for(id)
            .xy(rect.xy())
            .set(state.ids.background, ui);

        let rating = self.rating.map(|r| r.into()).unwrap_or(0);
        let [w, h] = ui.wh_of(id).unwrap_or(ui.window_dim());
        let star_size = (w / 5.0).min(h);

        let mut new_rating = None;

        for i in 0..5 {
            let is_filled = i < rating;
            let image = if is_filled { self.res.images.star_filled } else { self.res.images.star_outline };
            let b = widget::Button::image(image)
                .parent(id)
                .w_h(star_size, star_size)
                .image_color_with_feedback(ui.theme.shape_color);

            let b = if i == 0 {
                b.top_left_of(state.ids.background)
            } else {
                let prev = state.ids.stars[i - 1];
                b.align_top_of(prev).right_from(prev, 0.0)
            };

            for _click in b.set(state.ids.stars[i], ui) {
                let r = i + 1;
                new_rating = Some(if r == rating { None } else { Some(Rating::from(r)) });
                log::info!("Rating clicked: {} -> {:?}", i, new_rating);
            }
        }

        new_rating
    }
}