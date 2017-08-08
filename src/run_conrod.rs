use std;
use conrod;
use custom_widget::chatview;
use conrod::backend::glium::glium;
use app::Ids;
use greed_websocket::backend::websocket;
use find_folder;
use gui::set_ui;
use dyapplication::Application;
use libloading::Library;
use gui;
const WIN_W: u32 = 800;
const WIN_H: u32 = 600;
const LIB_PATH: &'static str = "target/debug/libtest_shared.so";

pub fn run<'a>(rust_logo: conrod::image::Id,
               event_rx: std::sync::mpsc::Receiver<ConrodMessage>,
               action_tx: std::sync::mpsc::Sender<chatview::Message>,
               render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
               window_proxy: glium::glutin::WindowProxy) {
    let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();

    // The `WidgetId` for our background and `Image` widgets.
    let ids = Ids::new(ui.widget_id_generator());
    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();
    let mut history: Vec<chatview::Message> = Vec::new();
    let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            "
            .to_owned();
    let mut name = "Kop".to_owned();
    let mut needs_update = true;
    let mut last_update = std::time::Instant::now();
    let mut app = Application(Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error)));

    let mut last_modified = std::fs::metadata(LIB_PATH).unwrap().modified().unwrap();
    'conrod: loop {
        if let Ok(Ok(modified)) = std::fs::metadata(LIB_PATH).map(|m| m.modified()) {
            if modified > last_modified {
                drop(app);
                app =
                    Application(Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error)));
                last_modified = modified;

            }
        }
        let sixteen_ms = std::time::Duration::from_millis(16);
        let now = std::time::Instant::now();
        let duration_since_last_update = now.duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }
        // Collect any pending events.
        let mut events = Vec::new();
        while let Ok(event) = event_rx.try_recv() {
            gui::update_state(rust_logo, event.clone(), &mut history);
            events.push(event);
        }

        // If there are no events pending, wait for them.
        if events.is_empty() || !needs_update {
            match event_rx.recv() {
                Ok(event) => {
                    gui::update_state(rust_logo, event.clone(), &mut history);
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


        // Instantiate the widgets.
        set_ui(ui.set_widgets(),
               &ids,
               &mut demo_text,
               &mut history,
               app.get_static_styles(),
               rust_logo,
               &mut name,
               action_tx.clone());
        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            if render_tx.send(primitives.owned()).is_err() {
                break 'conrod;
            }
            // Wakeup `winit` for rendering.
            window_proxy.wakeup_event_loop();
        }
    }
    // Load the Rust logo from our assets folder to use as an example image.

}

#[derive(Clone,Debug)]
pub enum ConrodMessage<'a> {
    Event(conrod::event::Input),
    Websocket(websocket::Message<'a>),
}
