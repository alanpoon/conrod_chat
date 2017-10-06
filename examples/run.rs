extern crate conrod;
extern crate conrod_chat;
extern crate futures;
extern crate hardback_server_lib;
extern crate toa_ping;
extern crate websocket;
extern crate find_folder;
extern crate image;
use conrod::backend::glium::glium::{self, glutin, Surface};
use conrod_chat::backend::websocket::client;
use hardback_server_lib::codec;
use std::sync::mpsc::{Sender, Receiver};
use futures::sync::mpsc;
use std::sync::{Arc, Mutex};
const WIN_W: u32 = 900;
const WIN_H: u32 = 600;
const CONNECTION: &'static str = "ws://127.0.0.1:8080";
pub struct GameApp {}

impl GameApp {
    pub fn new() -> Result<(), String> {
        let window = glutin::WindowBuilder::new();
        let context = glutin::ContextBuilder::new();
        let mut events_loop = glutin::EventsLoop::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();
        let rust_logo = load_rust_logo(&display);
        let (w, h) = (rust_logo.get_width(), rust_logo.get_height().unwrap());
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(rust_logo);

        let events_loop_proxy = events_loop.create_proxy();
        //<logic::game::ConrodMessage<OwnedMessage>>
        let (proxy_tx, proxy_rx) = std::sync::mpsc::channel();
        let (proxy_action_tx, proxy_action_rx) = mpsc::channel(2);
        let mut last_update = std::time::Instant::now();
        let s_tx = Arc::new(Mutex::new(proxy_action_tx));
        let s_rx = Arc::new(Mutex::new(proxy_action_rx));

        std::thread::spawn(move || {
            let mut connected = false;
            let mut last_update = std::time::Instant::now();

            while !connected {
                let sixteen_ms = std::time::Duration::new(20, 0);
                let now = std::time::Instant::now();
                let duration_since_last_update = now.duration_since(last_update);
                if duration_since_last_update < sixteen_ms {
                    std::thread::sleep(sixteen_ms - duration_since_last_update);
                }
                match toa_ping::run("www.google.com") {
                    Ok(_) => {
                        let (tx, rx) = mpsc::channel(3);
                        *(s_tx.lock().unwrap()) = tx;
                        match client::run(CONNECTION, proxy_tx.clone(), rx) {
                            Ok(_) => connected = true,
                            Err(err) => {
                                println!("reconnecting");
                                connected = false;
                            }
                        }
                    }
                    _ => {
                        connected = false;
                    }
                }
                last_update = std::time::Instant::now();
            }

        });

        let mut last_update = std::time::Instant::now();

        let mut events = Vec::new();
        let mut c = 0;
        'render: loop {
            let sixteen_ms = std::time::Duration::from_millis(500);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }
            events.clear();

            // Get all the new events since the last frame.
            events_loop.poll_events(|event| { events.push(event); });
            while let Ok(s) = proxy_rx.try_recv() {
                //update_state

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

                // Handle the input with the `Ui`.
                ui.handle_event(input);
                // Set the widgets.

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
    match GameApp::new() {
        Err(why) => println!("Error while running Hardback:\n{}", why),
        Ok(_) => (),
    }
}
fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(),
                                                                       image_dimensions);
    let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
    texture
}
