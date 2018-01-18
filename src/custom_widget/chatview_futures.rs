use conrod::{widget, Positionable, Widget, image, Sizeable, color, Labelable};
use conrod::color::Colorable;
use custom_widget::item_history;
use custom_widget::Message;
use futures::{Future, Sink};
use futures::sync::mpsc;

/// The type upon which we'll implement the `Widget` trait.
#[derive(WidgetCommon)]
pub struct ChatView<'a, T> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    pub lists: &'a mut Vec<Message>,
    pub text_edit: &'a mut String,
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    pub action_tx: mpsc::Sender<T>,
    pub image_id: Option<image::Id>,
    pub name: &'a String,
    pub closure: Box<fn(&String, &String) -> T>,
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
        text_edit_for_chat,
        text_edit_button_panel,
        text_edit_button,
    }
}

/// Represents the unique, cached state for our ChatView widget.
pub struct State {
    pub ids: Ids,
}

impl<'a, T> ChatView<'a, T> {
    /// Create a button context to be built upon.
    pub fn new(lists: &'a mut Vec<Message>,
               te: &'a mut String,
               image_id: Option<image::Id>,
               name: &'a String,
               action_tx: mpsc::Sender<T>,
               closure: Box<fn(&String, &String) -> T>)
               -> Self {
        ChatView {
            lists: lists,
            common: widget::CommonBuilder::default(),
            text_edit: te,
            style: Style::default(),
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
    builder_methods!{
        pub item_rect { style.item_rect = Some([f64;2]) }
    }
}

/// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
/// documentation for more details.
impl<'a, T> Widget for ChatView<'a, T> {
    /// The State struct that we defined above.
    type State = State;
    /// The Style struct that we defined using the `widget_style!` macro.
    type Style = Style;
    /// The event produced by instantiating the widget.
    ///
    /// `Some` when clicked, otherwise `None`.
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State { ids: Ids::new(id_gen) }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the button by handling any input that has occurred since the last
    /// update.
    fn update(self, args: widget::UpdateArgs<Self>) {
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
            let k =if w_can==0.0{
                "".to_owned()
            }else{
                self.text_edit.clone();
            };
        println!("136 {:?}",k.clone());
        let editz = widget::TextEdit::new(&k)
            .color(color::GREY)
            .padded_w_of(state.ids.text_edit_panel, 0.0)
            .mid_top_of(state.ids.text_edit_panel)
            .center_justify()
            .line_spacing(2.5)
            .restrict_to_height(false) // Let the height grow infinitely and scroll.
            .set(state.ids.text_edit_for_chat, ui);
             println!("144");
        for edit in editz {
            *self.text_edit = edit;
        }
        println!("148");
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
            let kc = k.clone();
            let g = (*self.closure)(self.name, &kc);
            self.action_tx
                .send(g)
                .wait()
                .unwrap();
            *self.text_edit = "".to_owned();
        };
        widget::Scrollbar::y_axis(state.ids.text_edit_panel)
            .auto_hide(false)
            .thickness(30.0)
            .color(color::LIGHT_GREY)
            .set(state.ids.text_edit_panel_scrollbar, ui);
        let num = self.lists.len();
        let (mut items, scrollbar) = widget::List::flow_down(num)
            .item_size(60.0)
            .scrollbar_thickness(30.0)
            .scrollbar_color(color::LIGHT_GREY)
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
        ()
    }
}
