use conrod::{self, widget, Colorable, Positionable, Widget, Sizeable, color};
use custom_widget::Message;
use std;
/// The type upon which we'll implement the `Widget` trait.
#[derive(WidgetCommon)]
pub struct ItemHistory<'a> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    pub message: &'a Message,
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    enabled: bool,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default="(color::BLUE,[200.0,30.0,2.0])")]
    pub item_rect: Option<(conrod::Color, [f64; 3])>, //w,h, pad bottom
    #[conrod(default="[20.0,20.0,10.0,10.0]")]
    pub item_image: Option<[f64; 4]>, // w,h,l,t
    #[conrod(default="(theme.label_color,theme.font_id,theme.font_size_medium,[100.0,50.0,22.0,5.0])")]
    pub item_text: Option<(conrod::Color,
                               Option<conrod::text::font::Id>,
                               conrod::FontSize,
                               [f64; 4])>, //RGB,w,h,l,t
}

widget_ids! {
    struct Ids {
        display_pic,
        name,
        text,
        rect,
        scrollbar
    }
}

/// Represents the unique, cached state for our ItemHistory widget.
pub struct State {
    ids: Ids,
}

impl<'a> ItemHistory<'a> {
    /// Create a button context to be built upon.
    pub fn new(message: &'a Message) -> Self {
        ItemHistory {
            message: message,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            enabled: true,
        }
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
    /// other Conrod configs, this returns self for chainability. Allow dead code
    /// because we never call this in the example.
    #[allow(dead_code)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }
    builder_methods!{
        pub item_rect { style.item_rect = Some((conrod::Color,[f64;3])) }
        pub item_image { style.item_image = Some([f64;4]) }
        pub item_text { style.item_text = Some((conrod::Color,Option<conrod::text::font::Id>,conrod::FontSize,[f64;4])) }
    }
}

/// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
/// documentation for more details.
impl<'a> Widget for ItemHistory<'a> {
    /// The State struct that we defined above.
    type State = State;
    /// The Style struct that we defined using the `widget_style!` macro.
    type Style = Style;
    /// The event produced by instantiating the widget.
    ///
    /// `Some` when clicked, otherwise `None`.
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State { ids: Ids::new(id_gen) }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the button by handling any input that has occurred since the last
    /// update.
    fn update(self, args: widget::UpdateArgs<Self>) -> std::option::Option<()> {
        let widget::UpdateArgs { id, state, ui, rect, style, .. } = args;
        // Finally, we'll describe how we want our widget drawn by simply instantiating the
        // necessary primitive graphics widgets.
        //
        let mut z = self.message.name.clone();
        z.push_str(": ");
        if let Some(k) = self.message.image_id {
            widget::Image::new(k)
                .top_left_with_margins_on(id,
                                          style.item_image(&ui.theme)[3],
                                          style.item_image(&ui.theme)[2])
                .w_h(style.item_image(&ui.theme)[0],
                     style.item_image(&ui.theme)[1])
                .set(state.ids.display_pic, ui);
        }
        widget::Text::new(&z)
            .top_left_with_margins_on(id,
                                      style.item_image(&ui.theme)[3],
                                      style.item_image(&ui.theme)[2] +
                                      style.item_image(&ui.theme)[0])
            .w(120.0)
            .set(state.ids.name, ui);
        let rect_w = rect.w() - 140.0 - style.item_image(&ui.theme)[2] +
                     style.item_image(&ui.theme)[0];
        widget::Rectangle::outline([rect_w, rect.h()])
            .right_from(state.ids.name, 0.0)
            .color(style.item_rect(&ui.theme).0)
            .scroll_kids_vertically()
            .set(state.ids.rect, ui);
        // Now we'll instantiate our label using the **Text** widget.
        let font_id = style.item_text(&ui.theme).1.or(ui.fonts.ids().next());
        widget::text_edit::TextEdit::new(&self.message.text)
            .font_id(font_id.unwrap())
            .top_left_with_margins_on(state.ids.rect,
                                      style.item_text(&ui.theme).3[3],
                                      style.item_text(&ui.theme).3[2])
            .font_size(style.item_text(&ui.theme).2)
            .color(style.item_text(&ui.theme).0)
            .restrict_to_height(false)
            .set(state.ids.text, ui);
        widget::Scrollbar::y_axis(state.ids.rect).auto_hide(false).set(state.ids.scrollbar, ui);

        Some(())
    }
}
