#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate conrod;
#[cfg(target_os="android")]
extern crate rusttype;
#[cfg(target_os="android")]
extern crate android_glue;
#[cfg(not(target_os="android"))]
extern crate find_folder;
extern crate conrod_chat;
extern crate futures;
extern crate hardback_server_lib;
extern crate toa_ping;
extern crate websocket;
extern crate image;
use conrod::{widget, color, Colorable, Widget, Positionable, Sizeable};
use conrod::backend::glium::glium::{self, glutin, Surface};
use conrod_chat::backend::websocket::client;
use conrod_chat::staticapplication as c_app;
use conrod_chat::custom_widget::chatview_futures;
use conrod_chat::chat;
use hardback_server_lib::codec;
use websocket::message::OwnedMessage;
use std::sync::mpsc::{Sender, Receiver};
use futures::sync::mpsc;
use std::sync::{Arc, Mutex};
//const CONNECTION: &'static str = "ws://127.0.0.1:8080";
const CONNECTION: &'static str = "ws://e404bb3c.ap.ngrok.io";
pub struct GameApp {}
widget_ids! {
    pub struct Ids {
         master,
         rect,
         chat
    }
}
pub mod support;
impl GameApp {
    pub fn new() -> Result<(), String> {
          let (proxy_tx, proxy_rx) = std::sync::mpsc::channel();
        let (proxy_action_tx, proxy_action_rx) = mpsc::channel(2);
        let s_tx = Arc::new(Mutex::new(proxy_action_tx));
        let s_rx = Arc::new(Mutex::new(proxy_action_rx));
        let (ss_tx, ss_rx) = (s_tx.clone(), s_rx.clone());
        std::thread::spawn(move || {
            let mut connected = false;
            let mut last_update = std::time::Instant::now();
            let mut c = 0;
            while !connected {
                let sixteen_ms = std::time::Duration::new(20, 0);
                let now = std::time::Instant::now();
                let duration_since_last_update = now.duration_since(last_update);
                if (duration_since_last_update < sixteen_ms) & (c > 0) {
                    std::thread::sleep(sixteen_ms - duration_since_last_update);
                }
                match toa_ping::run("www.google.com") {
                    Ok(_) => {
                                let (tx, rx) = mpsc::channel(3);
                        let mut ss_tx = ss_tx.lock().unwrap();
                        *ss_tx = tx;
                        drop(ss_tx);
                        match client::run_owned_message(CONNECTION, proxy_tx.clone(), rx) {
                            Ok(_) => connected = true,
                            Err(err) => {
                                println!("reconnecting");
                                connected = false;
                            }
                        }
                        
                        println!("Conrod www.google.com");
                        connected = true;
                    }
                    _ => {
                        connected = false;
                    }
                }
                last_update = std::time::Instant::now();
                c += 1;
            }

        });
        let window = glutin::WindowBuilder::new();
        let context =
            glium::glutin::ContextBuilder::new()
                .with_gl(glium::glutin::GlRequest::Specific(glium::glutin::Api::OpenGlEs, (3, 0)));
        let mut events_loop = glutin::EventsLoop::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
        // construct our `Ui`.
          let (w, h) = display.get_framebuffer_dimensions();
        let mut ui = conrod::UiBuilder::new([w as f64, h as f64]).build();
        println!("conrod ..ui");
         ui.fonts.insert(support::assets::load_font("fonts/NotoSans/NotoSans-Regular.ttf"));
          let rust_logo = load_rust_logo(&display);
          let (w, h) = (rust_logo.get_width(), rust_logo.get_height().unwrap());
        let mut image_map: conrod::image::Map<glium::texture::Texture2d> =
            conrod::image::Map::new();
          let rust_logo = image_map.insert(rust_logo);

        let events_loop_proxy = events_loop.create_proxy();
        //<logic::game::ConrodMessage<OwnedMessage>>
      
    
        let mut last_update = std::time::Instant::now();

        let mut events = Vec::new();
        let mut textedit = "".to_owned();
            let mut lobby_history = vec![];
        let mut c = 0;
        let mut ids = Ids::new(ui.widget_id_generator());

        'render: loop {
                 let ss_tx = s_tx.lock().unwrap();
                let proxy_action_tx = ss_tx.clone();
            let sixteen_ms = std::time::Duration::from_millis(500);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }
            events.clear();

            // Get all the new events since the last frame.
            events_loop.poll_events(|event| { events.push(event); });
                  while let Ok(msg) = proxy_rx.try_recv() {
                //update_state
                if let OwnedMessage::Text(z) = OwnedMessage::from(msg) {
                  
                    if let Ok(codec::ClientReceivedMsg { type_name,
                                                         location,
                                                         sender,
                                                         message,
                                                         .. }) =
                        codec::ClientReceivedMsg::deserialize_receive(&z) {
                            println!("location {:?},type_name {:?}, sender {:?}, message:{:?}",location,type_name,sender,message);
                        if let (Some(Some(_type_name)),
                                Some(Some(_location)),
                                Some(Some(_sender)),
                                Some(Some(_message))) =
                            (type_name.clone(), location, sender, message) {
                                  println!("_type_name {:?}",_type_name);
                            if _type_name == "chat" {

                                if _location == "lobby" {
                                    lobby_history.push(chat::message::Message {
                                                           image_id: Some(rust_logo),
                                                           name: _sender,
                                                           text: _message,
                                                       });
                                }
                            }
                        }
                    }
                }
            }
            
            // Process the events.
            for event in events.drain(..) {

                // Break from the loop upon `Escape` or closed window.
                match event.clone() {

                    glium::glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            glium::glutin::WindowEvent::Closed |
                            glium::glutin::WindowEvent::KeyboardInput {
                                input: glium::glutin::KeyboardInput {
                                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => break 'render,
                            _ => (),
                        }
                    }
                    _ => (),
                };

                // Use the `winit` backend feature to convert the winit event to a conrod input.
                let input = match conrod::backend::winit::convert_event(event, &display) {
                    None => continue,
                    Some(input) => input,
                };
                let f = "alan".to_owned();
                // Handle the input with the `Ui`.
                ui.handle_event(input);
                // Set the widgets.
                let ui = &mut ui.set_widgets();
                widget::Canvas::new().color(color::LIGHT_BLUE).set(ids.master, ui);
                      let k = chatview_futures::ChatView::new(&mut lobby_history,
                                                        &mut textedit,
                                                        get_chat_styles(),
                                                        Some(rust_logo),
                                                        &f,
                                                        proxy_action_tx.clone(),
                                                        Box::new(process))
                        .middle()
                        .padded_wh_of(ids.master, 100.0)
                        .set(ids.chat, ui);
                        
            }

            let primitives = ui.draw();

            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();

            target.finish().unwrap();
            c += 1;
        }
        Ok(())
    }
}
fn main() {
    println!("conrod ..main");
    match GameApp::new() {
        Err(why) => println!("Error while running Hardback:\n{}", why),
        Ok(_) => (),
    }
}
fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
    let rgba_image = support::assets::load_image("images/rust.png").to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(),
                                                                       image_dimensions);
    let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
    texture
}
fn process(name: &String, text: &String) -> OwnedMessage {
    let g = json!({
    "type":"chat",
  "message": text,
  "location":"lobby"
});
    OwnedMessage::Text(g.to_string())
}

fn get_chat_styles() -> c_app::Static_Style {
    c_app::Application::default().get_static_styles()
}
