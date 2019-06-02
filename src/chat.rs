use conrod_core;
use std::time::Instant;

pub mod message {
    pub use custom_widget::Message;
}
#[cfg(feature="keypad")]
pub use conrod_keypad::english;
#[derive(Clone,Debug)]
pub enum ConrodMessage {
    Event(Instant, conrod_core::event::Input),
    Thread(Instant),
}
