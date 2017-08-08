use conrod::{self, widget, Colorable, Positionable, Widget, Sizeable, color};
#[cfg(feature="hotload")]
use dyapplication as application;
#[cfg(not(feature="hotload"))]
use staticapplication as application;
use custom_widget::chatview::Message;
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
    pub static_style: &'a application::Static_Style,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    /// Color of the button's label.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod::Color>,
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<conrod::Color>,
    /// Font size of the button's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<conrod::FontSize>,
    /// Specify a unique font for the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<conrod::text::font::Id>>,
}

widget_ids! {
    struct Ids {
        display_pic,
        name,
        text,
        rect,
    }
}

/// Represents the unique, cached state for our ItemHistory widget.
pub struct State {
    ids: Ids,
}

impl<'a> ItemHistory<'a> {
    /// Create a button context to be built upon.
    pub fn new(message: &'a Message, static_style: &'a application::Static_Style) -> Self {
        ItemHistory {
            message: message,
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            enabled: true,
            static_style: static_style,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: conrod::text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
    /// other Conrod configs, this returns self for chainability. Allow dead code
    /// because we never call this in the example.
    #[allow(dead_code)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
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
        let widget::UpdateArgs { id, state, rect, mut ui, style, .. } = args;
        // Finally, we'll describe how we want our widget drawn by simply instantiating the
        // necessary primitive graphics widgets.
        //
        let static_style = self.static_style;
        let application::RGB(rr, rg, rb, ra) = static_style.rect.0;
        widget::Rectangle::outline([static_style.rect.1, static_style.rect.2])
            .align_left_of(id)
            .color(color::Color::Rgba(rr, rg, rb, ra))
            .set(state.ids.rect, ui);
        if let Some(k) = self.message.image_id{
            widget::Image::new(k)
            .top_left_with_margins_on(state.ids.rect, static_style.image.4, static_style.image.3)
            .w_h(static_style.image.1, static_style.image.2)
            .set(state.ids.display_pic, ui);
        }
      
        let mut z = self.message.name.clone();
        z.push_str(" : ");
        z.push_str(&self.message.text);
        // Now we'll instantiate our label using the **Text** widget.
        let application::RGB(tr, tg, tb, ta) = static_style.text.1;
        let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
        widget::Text::new(&z)
            .and_then(font_id, widget::Text::font_id)
            .top_left_with_margins_on(state.ids.rect, static_style.text.5, static_style.text.4)
            .font_size(static_style.text.0)
            .color(color::Color::Rgba(tr, tg, tb, ta))
            .set(state.ids.text, ui);
        Some(())
    }
}