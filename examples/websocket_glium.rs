#[cfg(all(feature="backend_glium_winit",feature="web_socket"))]
#[macro_use]
mod support;
#[cfg(all(feature="backend_glium_winit", feature="web_socket"))]
#[macro_use]
extern crate conrod;
#[macro_use]
extern crate serde_derive;
fn main() {
    feature::main();
}

mod app {
    extern crate serde_json;
    extern crate serde;
    use self::serde::{Deserialize, Deserializer};
    use support::app_macros;
    fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
        where D: Deserializer<'de>,
              T: Deserialize<'de>
    {
        Ok(Some(Option::deserialize(deserializer)?))
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct TableInfo {
        numberOfPlayer: i32,
        players: Vec<String>,
    }
    #[derive(Serialize, Deserialize,Debug, Clone)]
    pub struct Player {
        name: String,
        cash: i32,
        cars: i32,
        guns: i32,
        keys: i32,
        hearts: i32,
        bottles: i32,
        wrenches: i32,
        holdings: Vec<i32>,
        thugs: Vec<i32>,
        actions: Vec<i32>,
    }
    send_msg_macro!{
        (newTable,set_newTable,bool),
        (ready,set_ready,bool),
        (joinTable,set_joinTable,i32),
        (changePlayers,set_changePlayer,i32),
        (leaveTable,set_leaveTable,bool),
        (joinLobby,set_joinLobby,bool),
        (namechange,set_namechange,String),
        (chat,set_chat,String),
        (location,set_location,String),
        (greedcommand,set_greedcommand,i32),
    }
    receive_msg_macro!{
        rename:{
        },optional:{
        (tables,set_tables,Vec<TableInfo>),
        (players,set_players,Vec<Player>),
        (request,set_request,String),
        (reason,set_reason,String),
        (optional,set_optional,bool),
        (location,set_location,String),
        (sender,set_sender,String),
        (message,set_message,String)
        },rename_optional:{ (type_name,set_type_name,String,"type"),},else:{}
    }
}
#[cfg(all(feature="backend_glium_winit",feature="web_socket"))]
mod feature {
    extern crate conrod_chat;
    extern crate find_folder;
    extern crate image;
    extern crate futures;
    extern crate websocket;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    const WIN_W: u32 = 600;
    const WIN_H: u32 = 800;
    const CONNECTION: &'static str = "ws://ec2-35-157-160-241.eu-central-1.compute.amazonaws.com:8080/greed";
    use feature::futures::{Future, Sink};
    use self::conrod_chat::backend::websocket::client;
    use self::conrod_chat::chat;
    use std;
    use app;

    pub fn main() {
        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Conrod with glium!")
            .with_dimensions(WIN_W, WIN_H);
        let context = glium::glutin::ContextBuilder::new().with_vsync(true).with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
        fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
            let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
            let path = assets.join("images/rust.png");
            let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
            let image_dimensions = rgba_image.dimensions();
            let raw_image =
                glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(),
                                                                   image_dimensions);
            let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
            texture
        }
        fn draw(display: &glium::Display,
                renderer: &mut conrod::backend::glium::Renderer,
                image_map: &conrod::image::Map<glium::Texture2d>,
                primitives: &conrod::render::OwnedPrimitives) {
            renderer.fill(display, primitives.walk(), &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        // Create our `conrod::image::Map` which describes each of our widget->image mappings.
        // In our case we only have one image, however the macro may be used to list multiple.
        let rust_logo = load_rust_logo(&display);
        let (w, h) = (rust_logo.get_width(), rust_logo.get_height().unwrap());
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(rust_logo);

        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (render_tx, render_rx) = std::sync::mpsc::channel();
        // This window proxy will allow conrod to wake up the `winit::Window` for rendering.
        let events_loop_proxy = events_loop.create_proxy();
        let mut last_update = std::time::Instant::now();
        let (futures_tx, futures_rx) = futures::sync::mpsc::channel(1);
        let futures_tx_clone = futures_tx.clone();
        let futures_tx_clone2 = futures_tx.clone();
        let (proxy_action_tx, proxy_action_rx) = std::sync::mpsc::channel(); //chatview::Message
        std::thread::spawn(move || {
            let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();
            // Add a `Font` to the `Ui`'s `font::Map` from file.
            let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
            let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
            ui.fonts.insert_from_file(font_path).unwrap();
            let cj =
                chat::ChatInstance::new("Alan".to_owned(), Box::new(move |h: &mut Vec<chat::message::Message>,
                                         conrod_msg: chat::ConrodMessage<websocket::Message>| {
                    if let chat::ConrodMessage::Socket(j) = conrod_msg.clone() {
                        if let websocket::OwnedMessage::Text(z) = websocket::OwnedMessage::from(j) {
                            if let Ok(app::ReceivedMsg { type_name,
                                                         tables,
                                                         players,
                                                         request,
                                                         reason,
                                                         optional,
                                                         location,
                                                         sender,
                                                         message }) =
                                app::ReceivedMsg::deserialize_receive(&z) {
                                if let (Some(Some(_type_name)),
                                        Some(Some(_location)),
                                        Some(Some(_sender)),
                                        Some(Some(_message))) =
                                    (type_name, location, sender, message) {
                                    if _type_name == "chat" {
                                        if _location == "lobby" {
                                            h.push(chat::message::Message {
                                                       image_id: Some(rust_logo),
                                                       name: _sender,
                                                       text: _message,
                                                   });
                                        }
                                        if _location == "table" {}
                                    }


                                }



                            }
                        }

                    }
                }))
                        .set_image_id(rust_logo)
                        .run(&mut ui,
                             event_rx,
                             proxy_action_tx,
                             render_tx,
                             events_loop_proxy);
        });
        std::thread::spawn(move || {
            let o = "{chat:'hello',location:'lobby'}";
            let mut c = 0;
            'update: loop {
                if c > 2 {
                    break 'update;
                }
                futures_tx_clone.clone()
                    .send(websocket::Message::text(o))
                    .wait()
                    .unwrap();
                let five_sec = std::time::Duration::from_secs(5);
                std::thread::sleep(five_sec);
                c += 1;
            }
        });
        let (proxy_tx, proxy_rx) = std::sync::mpsc::channel();
        let event_tx_clone_2 = event_tx.clone();
        std::thread::spawn(move || 'proxy: loop {
                               //send to conrod
                               while let Ok(s) = proxy_rx.try_recv() {
                                   event_tx_clone_2.send(chat::ConrodMessage::Socket(s)).unwrap();
                               }
                               // send to Websocket
                               while let Ok(s) = proxy_action_rx.try_recv() {
                                   let chat::message::Message { text, .. } = s;
                                   let kl =
                    app::SendMsg::new().set_chat(text).set_location("lobby".to_owned()).clone();
                                   if let Ok(_text) = app::SendMsg::serialize_send(kl) {
                                       println!("_text json: {:?}", _text);
                                       futures_tx_clone2.clone()
                                           .send(websocket::Message::text(_text))
                                           .wait()
                                           .unwrap();
                                   }
                               }
                           });
        std::thread::spawn(move || { client::run(CONNECTION, proxy_tx, futures_rx); });

        let mut closed = false;
        while !closed {

            // We don't want to loop any faster than 60 FPS, so wait until it has been at least
            // 16ms since the last yield.
            let sixteen_ms = std::time::Duration::from_millis(16);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }
            events_loop.run_forever(|event| {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    event_tx.send(chat::ConrodMessage::Event(event)).unwrap();
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::Closed |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => {
                            closed = true;
                            return glium::glutin::ControlFlow::Break;
                        },
                        // We must re-draw on `Resized`, as the event loops become blocked during
                        // resize on macOS.
                        glium::glutin::WindowEvent::Resized(..) => {
                            if let Some(primitives) = render_rx.iter().next() {
                                draw(&display, &mut renderer, &image_map, &primitives);
                            }
                        },
                        _ => {},
                    },
                    glium::glutin::Event::Awakened => return glium::glutin::ControlFlow::Break,
                    _ => (),
                }

                glium::glutin::ControlFlow::Continue
});

            // Draw the most recently received `conrod::render::Primitives` sent from the `Ui`.
            if let Some(primitives) = render_rx.try_iter().last() {
                draw(&display, &mut renderer, &image_map, &primitives);
            }

            last_update = std::time::Instant::now();
        }



    }
}
#[cfg(not(all(feature="backend_glium_winit",feature="web_socket")))]
mod feature {
    pub fn main() {
        println!("This example requires the `backend_glium_winit` and `web_socket` features. \
                 Try running `cargo run --release --features=\"backend_glium_winit web_socket\" --example <example_name>`");
    }
}
