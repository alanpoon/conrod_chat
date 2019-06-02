#[macro_use]
extern crate conrod_core;
extern crate conrod_derive;
extern crate conrod_glium;
extern crate conrod_keypad;
extern crate image;
extern crate glium;
#[cfg(target_os="android")]
extern crate rusttype;
#[cfg(target_os="android")]
extern crate android_glue;
#[cfg(not(target_os="android"))]
extern crate find_folder;

pub mod support;
use conrod_core::{widget, color, Colorable, Widget, Positionable, Sizeable};
use glium::Surface;
use conrod_keypad::custom_widget::{text_edit, keypad};
use conrod_keypad::english;
use std::time::Instant;

widget_ids! {
    pub struct Ids {
         master,
         keyboard,
         image,
         text_edit
    }
}
pub enum ConrodMessage {
    Event(Instant, conrod_core::event::Input),
    Thread(Instant),
}
const WIDTH: u32 = 600;
const HEIGHT: u32 = 420;
fn main() {
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
    ui.fonts.insert(support::assets::load_font("fonts/NotoSans/NotoSans-Regular.ttf"));
    let rust_logo = load_image(&display.0, "images/rust.png");
    let mut image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();
    let rust_logo = image_map.insert(rust_logo);
    let events_loop_proxy = events_loop.create_proxy();
    let mut ids = Ids::new(ui.widget_id_generator());
    let mut demo_text_edit = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
            finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
            fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
            Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
            Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
            magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();
    let mut last_update = std::time::Instant::now();
    let mut last_update_sys = std::time::SystemTime::now();
    let mut c = 0;
    let mut keypadvariant = keypad::KeyPadVariant::Letter(1);
    let mut captured_event: Option<ConrodMessage> = None;
    let sixteen_ms = std::time::Duration::from_millis(100);
    let english_tuple = english::populate();
    'render: loop {
        let mut to_break = false;
        let mut to_continue = false;
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
                        } => {to_break=true;},
                        _ => (),
                    }
                }
                _ => (),
            };
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
                            &mut demo_text_edit,
                            &mut keypadvariant,
                            &english_tuple,
                            &mut ids);
            }
            Some(ConrodMessage::Thread(t)) => {
                let mut ui = ui.set_widgets();
                set_widgets(&mut ui,
                            &mut demo_text_edit,
                            &mut keypadvariant,
                            &english_tuple,
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

        let primitives = ui.draw();
        renderer.fill(&display.0, primitives, &image_map);
        let mut target = display.0.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        renderer.draw(&display.0, &mut target, &image_map).unwrap();
        target.finish().unwrap();
    }
}
fn load_image(display: &glium::Display, path: &str) -> glium::texture::Texture2d {
    let rgba_image = support::assets::load_image(path).to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(),
                                                                       image_dimensions);
    let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
    texture
}
fn set_widgets(ui: &mut conrod_core::UiCell,
               demo_text_edit: &mut String,
               keypadvariant: &mut keypad::KeyPadVariant,
               english_tuple: &(Vec<english::KeyButton>,
                                Vec<english::KeyButton>,
                                english::KeyButton),
               ids: &mut Ids) {
    widget::Canvas::new().color(color::LIGHT_BLUE).set(ids.master, ui);
    let (editz, keypad_bool) = text_edit::TextEdit::new(demo_text_edit,ids.master,&english_tuple)
            .color(color::WHITE)
            .padded_w_of(ids.master, 20.0)
            .mid_top_of(ids.master)
            .center_justify()
            .line_spacing(2.5)
            .restrict_to_height(false) // Let the height grow infinitely and scroll.
            .set(ids.text_edit, ui) ;
        for edit in editz{
         *demo_text_edit = edit;
        }

}
