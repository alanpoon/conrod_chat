use conrod_core::{self, widget, Colorable, Labelable, Positionable, Widget, Sizeable, color};
use custom_widget::item_history;
use conrod_keypad::custom_widget::text_edit::TextEdit;
use conrod_keypad::english;
use custom_widget::Message;
use std::sync::mpsc;
/// The type upon which we'll implement the `Widget` trait.
#[derive(WidgetCommon)]
pub struct ChatView<'a> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    pub lists: &'a mut Vec<Message>,
    pub text_edit: &'a mut String,
    pub master_id: widget::Id,
    pub english_tuple: &'a (Vec<english::KeyButton>, Vec<english::KeyButton>, english::KeyButton),
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    pub action_tx: mpsc::Sender<String>,
    pub image_id: Option<conrod_core::image::Id>,
    pub name: &'a String,
    pub closure: Box<fn(&str, &str) -> String>,
    enabled: bool,
}


#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default="[200.0,30.0]")]
    pub item_rect: Option<[f64; 2]>, //w,h, pad bottom
}

widget_ids! {
    pub struct Ids {
        chat_canvas,
        message_panel,
        history_list,
        text_edit_body,
        text_edit_panel,
        text_edit_panel_scrollbar,
        text_edit,
        text_edit_button_panel,
        text_edit_button,
    }
}

/// Represents the unique, cached state for our ChatView widget.
pub struct State {
    pub ids: Ids,
}

impl<'a> ChatView<'a> {
    /// Create a button context to be built upon.
    pub fn new(lists: &'a mut Vec<Message>,
               te: &'a mut String,
               master_id: widget::Id,
               english_tuple: &'a (Vec<english::KeyButton>,
                                   Vec<english::KeyButton>,
                                   english::KeyButton),
               image_id: Option<conrod_core::image::Id>,
               name: &'a String,
               action_tx: mpsc::Sender<String>,
               closure: Box<fn(&str, &str) -> String>)
               -> Self {
        ChatView {
            lists: lists,
            common: widget::CommonBuilder::default(),
            text_edit: te,
            style: Style::default(),
            master_id: master_id,
            english_tuple: english_tuple,
            image_id: image_id,
            name: name,
            action_tx: action_tx,
            closure: closure,
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
}

/// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
/// documentation for more details.
impl<'a> Widget for ChatView<'a> {
    /// The State struct that we defined above.
    type State = State;
    /// The Style struct that we defined using the `widget_style!` macro.
    type Style = Style;
    /// The event produced by instantiating the widget.
    ///
    /// `Some` when clicked, otherwise `None`.
    type Event = bool;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State { ids: Ids::new(id_gen) }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the button by handling any input that has occurred since the last
    /// update.
    fn update(self, args: widget::UpdateArgs<Self>) -> bool {
        let widget::UpdateArgs { id, state, ui, style, .. } = args;
        // Finally, we'll describe how we want our widget drawn by simply instantiating the
        // necessary primitive graphics widgets.
        //
        let can = ui.rect_of(id).unwrap();
        let w_can = can.w();
        let h_can = can.h();
        widget::Canvas::new()
            .flow_down(&[(state.ids.message_panel,
                          widget::Canvas::new().color(color::GREEN).pad_bottom(20.0)),
                         (state.ids.text_edit_body,
                          widget::Canvas::new()
                              .length(h_can * 0.2)
                              .flow_right(&[(state.ids.text_edit_panel,
                                             widget::Canvas::new()
                                                 .scroll_kids_vertically()
                                                 .color(color::DARK_CHARCOAL)
                                                 .length(w_can * 0.7)),
                                            (state.ids.text_edit_button_panel,
                                             widget::Canvas::new()
                                                 .color(color::DARK_CHARCOAL))]))])
            .middle_of(id)
            .set(state.ids.chat_canvas, ui);

        let k = self.text_edit;
        let (editz, keypad_bool) = TextEdit::new(k,self.master_id,self.english_tuple)
            .padded_w_of(state.ids.text_edit_panel, 20.0)
            .mid_top_of(state.ids.text_edit_panel)
            .center_justify()
            .line_spacing(2.5)
            .restrict_to_height(false) // Let the height grow infinitely and scroll.
            .set(state.ids.text_edit, ui);
        for edit in editz {
            *k = edit;
        }
        let button_panel = ui.rect_of(state.ids.text_edit_button_panel).unwrap();
        let w_button_panel = button_panel.w();
        let h_button_panel = button_panel.h();
        if widget::Button::new()
               .color(color::GREY)
               .padded_w_of(state.ids.text_edit_button_panel, 0.2 * w_button_panel)
               .padded_h_of(state.ids.text_edit_button_panel, 0.2 * h_button_panel)
               .label("Enter")
               .middle_of(state.ids.text_edit_button_panel)
               .set(state.ids.text_edit_button, ui)
               .was_clicked() {
            let g = (*self.closure)(self.name, k);
            self.action_tx.send(g).unwrap();
            *k = "".to_owned();
        };
        widget::Scrollbar::y_axis(state.ids.text_edit_panel)
            .auto_hide(true)
            .set(state.ids.text_edit_panel_scrollbar, ui);
        let num = self.lists.len();
        let (mut items, scrollbar) = widget::List::flow_down(num)
            .scrollbar_on_top()
            .middle_of(state.ids.message_panel)
            .wh_of(state.ids.message_panel)
            .set(state.ids.history_list, ui);

        if let Some(s) = scrollbar {
            s.set(ui)
        }
        let mut it_j = self.lists.iter();
        while let (Some(a), Some(item)) = (it_j.next(), items.next(ui)) {
            let cb = item_history::ItemHistory::new(&a).w_h(style.item_rect(&ui.theme)[0],
                                                            style.item_rect(&ui.theme)[1]);
            item.set(cb, ui);
        }
        keypad_bool
    }
}
