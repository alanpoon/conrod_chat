use custom_widget::chatview;
use conrod::{self, color, widget, Colorable, Positionable, Widget};
use conrod::backend::glium::glium;
use std;
use app::Ids;
use std::sync::mpsc;
#[cfg(feature="hotload")]
use dyapplication as application;
#[cfg(not(feature="hotload"))]
use staticapplication as application;

const LIB_PATH: &'static str = "target/debug/libtest_shared.so";
pub struct ChatInstance<T: Clone> {
    pub textedit: String,
    pub image_id: Option<conrod::image::Id>,
    pub name: String,
    pub update_closure: Box<Fn(&mut Vec<chatview::Message>, ConrodMessage<T>)>,
}
pub mod message {
    pub use custom_widget::chatview::Message;
}
impl<T> ChatInstance<T>
    where T: Clone
{
    pub fn new(name: String,
               f: Box<Fn(&mut Vec<chatview::Message>, ConrodMessage<T>)>)
               -> ChatInstance<T> {
        ChatInstance {
            textedit: "".to_owned(),
            image_id: None,
            name: name,
            update_closure: f,
        }
    }
    pub fn set_image_id(&mut self, s: conrod::image::Id) -> &mut Self {
        self.image_id = Some(s);
        self
    }
    pub fn run(&mut self,
               ui: &mut conrod::Ui,
               history:&mut Vec<chatview::Message>,
               event_rx: std::sync::mpsc::Receiver<ConrodMessage<T>>,
               action_tx: std::sync::mpsc::Sender<String>,
               render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
               events_loop_proxy: glium::glutin::EventsLoopProxy) {

        // The `WidgetId` for our background and `Image` widgets.
        let ids = Ids::new(ui.widget_id_generator());
        let mut needs_update = true;
        let mut last_update = std::time::Instant::now();
        let mut app = application::Application::new(LIB_PATH);
        'conrod: loop {
            application::Application::in_loop(&mut app, LIB_PATH, &mut last_update);
            let sixteen_ms = std::time::Duration::from_millis(16);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }
            // Collect any pending events.
            let mut events = Vec::new();
            while let Ok(event) = event_rx.try_recv() {
                self.update_state(history, event.clone());
                events.push(event);
            }
            if events.is_empty() || !needs_update {
                match event_rx.recv() {
                    Ok(event) => {
                        self.update_state(history, event.clone());
                        events.push(event);
                    }
                    Err(_) => break 'conrod,
                };
            }

            needs_update = false;
            // Input each event into the `Ui`.
            for event in events {
                if let ConrodMessage::Event(e) = event {
                    ui.handle_event(e);
                }
                needs_update = true;
            }
            self.set_ui(ui.set_widgets(),
                        &ids,
                        history,
                        app.get_static_styles(),
                        action_tx.clone());
            if let Some(primitives) = ui.draw_if_changed() {
                if render_tx.send(primitives.owned()).is_err() ||
                   events_loop_proxy.wakeup().is_err() {
                    break 'conrod;
                }
            }
        }
    }
    fn update_state(&self, mut o: &mut Vec<chatview::Message>, conrod_msg: ConrodMessage<T>) {
        (*self.update_closure)(o, conrod_msg);
    }
    fn set_ui(&mut self,
              ref mut ui: conrod::UiCell,
              ids: &Ids,
              history: &mut Vec<chatview::Message>,
              styles: application::Static_Style,
              action_tx: mpsc::Sender<String>) {
        widget::Canvas::new().color(color::LIGHT_BLUE).set(ids.master, ui);
        // Instantiate the `Image` at its full size in the middle of the window.
        chatview::ChatView::new(history,
                                &mut self.textedit,
                                styles,
                                self.image_id,
                                &self.name,
                                action_tx,
                                Box::new(process))
                .middle_of(ids.master)
                .set(ids.chatview, ui);
    }
}
fn process(name:&str,text:&str)->String{
        let g = format!("{{chat:{},location:'lobby'}}",text);
        g
}
#[derive(Clone,Debug)]
pub enum ConrodMessage<T: Clone> {
    Event(conrod::event::Input),
    Socket(T),
}