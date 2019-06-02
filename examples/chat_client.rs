#[macro_use]
extern crate serde_json;
extern crate conrod_core;
extern crate conrod_glium;
#[cfg(target_os="android")]
extern crate rusttype;
#[cfg(target_os="android")]
extern crate android_glue;
#[cfg(not(target_os="android"))]
extern crate find_folder;
extern crate conrod_chat;
extern crate futures;
extern crate hardback_codec;
extern crate toa_ping;
extern crate websocket;
extern crate image;
extern crate glium;
// run with --features "keypad"
use conrod_core::{widget, color, Colorable, Widget, Positionable, Sizeable};
use glium::Surface;
use conrod_chat::backend::websocket::client;
use conrod_chat::custom_widget::chatview_futures;
use conrod_chat::chat;
use conrod_chat::chat::ConrodMessage;
use hardback_codec::codec;
use websocket::message::OwnedMessage;
use std::sync::mpsc::{Sender, Receiver};
use futures::sync::mpsc;
use std::sync::{Arc, Mutex};
const CONNECTION: &'static str = "ws://0.0.0.0:8080";

pub struct GameApp {}
widget_ids! {
    pub struct Ids {
         master,
         rect,
         chat_canvas,
         chat,
         keypad,
         keypad_canvas
    }
}
pub mod support;
const WIDTH: u32 = 600;
const HEIGHT: u32 = 420;
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
                println!("connected {:?}", connected);
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
                        match client::run_owned_message(CONNECTION.to_owned(), proxy_tx.clone(), rx) {
                            Ok(_) => connected = true,
                            Err(err) => {
                                println!("reconnecting");
                                connected = false;
                            }
                        }
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
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Hello Conrod!")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let display = support::glium_wrapper::GliumDisplayWinitWrapper(display);
        let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();
        // construct our `Ui`.
        let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
        println!("conrod ..ui");
        ui.fonts.insert(support::assets::load_font("fonts/NotoSans/NotoSans-Regular.ttf"));
        let rust_logo = load_image(&display.0, "images/rust.png");
        let mut image_map: conrod_core::image::Map<glium::texture::Texture2d> =
            conrod_core::image::Map::new();
        let rust_logo = image_map.insert(rust_logo);
        let events_loop_proxy = events_loop.create_proxy();
        //<logic::game::ConrodMessage<OwnedMessage>>
        let mut last_update = std::time::Instant::now();
        let mut last_update_sys = std::time::SystemTime::now();
        let mut demo_text_edit = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
            finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
            fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
            Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
            Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
            magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();
        
        //let mut demo_text_edit = "".to_owned();
        let mut lobby_history = vec![];
        let mut c = 0;
        let mut ids = Ids::new(ui.widget_id_generator());
        let name = "alan".to_owned();
        let sixteen_ms = std::time::Duration::from_millis(100);
        let mut captured_event: Option<ConrodMessage> = None;
        let mut keypad_bool = false;
        'render: loop {
            let mut to_break = false;
            let mut to_continue = false;
            let ss_tx = s_tx.lock().unwrap();
            let proxy_action_tx = ss_tx.clone();
            events_loop.poll_events(|event| {
                match event.clone() {
                    glium::glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            glium::glutin::WindowEvent::CloseRequested |
                            glium::glutin::WindowEvent::KeyboardInput {
                                input: glium::glutin::KeyboardInput {
                                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => {to_break=true;}
                            _ => (),
                        }
                    }
                    _ => {}
                }
                let input = match conrod_winit::convert_event(event, &display) {
                    None => {
                        to_continue = true;
                    }
                    Some(input) => {
                        let d = std::time::Instant::now();
                        captured_event = Some(ConrodMessage::Event(d, input));
                    }
                };
            });
            if to_break {
                break 'render;
            }
            if to_continue {
                continue;
            }
            match captured_event {
                Some(ConrodMessage::Event(_, ref input)) => {
                    ui.handle_event(input.clone());
                    let mut ui = ui.set_widgets();
                    set_widgets(&mut ui,
                                [WIDTH as f64, HEIGHT as f64],
                                &mut lobby_history,
                                &mut demo_text_edit,
                                &mut keypad_bool,
                                &name,
                                Some(rust_logo),
                                proxy_action_tx.clone(),
                                &mut ids);
                }
                Some(ConrodMessage::Thread(t)) => {
                    let mut ui = ui.set_widgets();
                    set_widgets(&mut ui,
                                [WIDTH as f64, HEIGHT as f64],
                                &mut lobby_history,
                                &mut demo_text_edit,
                                &mut keypad_bool,
                                &name,
                                Some(rust_logo),
                                proxy_action_tx.clone(),
                                &mut ids);
                }
                None => {
                    let now = std::time::Instant::now();
                    let duration_since_last_update = now.duration_since(last_update);
                    if duration_since_last_update < sixteen_ms {
                        std::thread::sleep(sixteen_ms - duration_since_last_update);
                    }
                    let t = std::time::Instant::now();
                    captured_event = Some(ConrodMessage::Thread(t));
                }
            }
            while let Ok(msg) = proxy_rx.try_recv() {
                //update_state
                if let OwnedMessage::Text(z) = OwnedMessage::from(msg) {

                    if let Ok(codec::ClientReceivedMsg { type_name,
                                                         location,
                                                         sender,
                                                         message,
                                                         .. }) =
                        codec::ClientReceivedMsg::deserialize_receive(&z) {
                        println!("location {:?},type_name {:?}, sender {:?}, message:{:?}",
                                 location,
                                 type_name,
                                 sender,
                                 message);
                        if let (Some(Some(_type_name)),
                                Some(Some(_location)),
                                Some(Some(_sender)),
                                Some(Some(_message))) =
                            (type_name.clone(), location, sender, message) {
                            println!("_type_name {:?}", _type_name);
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

        let primitives = ui.draw();
        renderer.fill(&display.0, primitives, &image_map);
        let mut target = display.0.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(&display.0, &mut target, &image_map).unwrap();
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

fn load_image(display: &glium::Display, path: &str) -> glium::texture::Texture2d {
    let rgba_image = support::assets::load_image(path).to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(),
                                                                       image_dimensions);
    let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
    texture
}
#[cfg(feature="keypad")]
fn set_widgets(ui: &mut conrod_core::UiCell,
               dimension: [f64; 2],
               lobby_history: &mut Vec<chat::message::Message>,
               text_edit: &mut String,
               keypad_bool: &mut bool,
               name: &String,
               rust_logo: Option<conrod_core::image::Id>,
               action_tx: mpsc::Sender<OwnedMessage>,
               ids: &mut Ids) {
    use conrod_chat::chat::{ english};
    let english_tuple = english::populate();

    let (keypad_length, _) = if *keypad_bool {
        (dimension[1] * 0.375, 400.0)
    } else {
        (0.0, 700.0)
    };
    widget::Canvas::new()
        .flow_down(&[(ids.chat_canvas, widget::Canvas::new().color(color::LIGHT_BLUE)),
                     (ids.keypad_canvas,
                      widget::Canvas::new().length(keypad_length).color(color::LIGHT_BLUE))])
        .set(ids.master, ui);
    let keypad_bool_ = chatview_futures::ChatView::new(lobby_history,
                                                       text_edit,
                                                       ids.master,
                                                       &english_tuple,
                                                       rust_logo,
                                                       name,
                                                       action_tx,
                                                       Box::new(process))
            .middle_of(ids.chat_canvas)
            .padded_wh_of(ids.chat_canvas, 0.0)
            .set(ids.chat, ui);
    *keypad_bool = keypad_bool_;
}
#[cfg(not(feature="keypad"))]
fn set_widgets(ui: &mut conrod_core::UiCell,
               dimension: [f64; 2],
               lobby_history: &mut Vec<chat::message::Message>,
               text_edit: &mut String,
               keypad_bool: &mut bool,
               name: &String,
               rust_logo: Option<conrod_core::image::Id>,
               _keypad:conrod_core::image::Id,
               action_tx: mpsc::Sender<OwnedMessage>,
               ids: &mut Ids) {
    let (keypad_length, _) = if *keypad_bool {
        (dimension[1] * 0.375, 400.0)
    } else {
        (0.0, 700.0)
    };
    widget::Canvas::new()
        .flow_down(&[(ids.chat_canvas, widget::Canvas::new().color(color::LIGHT_BLUE)),
                     (ids.keypad_canvas,
                      widget::Canvas::new().length(keypad_length).color(color::LIGHT_BLUE))])
        .set(ids.master, ui);
    chatview_futures::ChatView::new(lobby_history,
                                                       text_edit,
                                                       rust_logo,
                                                       name,
                                                       action_tx,
                                                       Box::new(process))
            .middle_of(ids.chat_canvas)
            .padded_wh_of(ids.chat_canvas, 0.0)
            .set(ids.chat, ui);
    
}