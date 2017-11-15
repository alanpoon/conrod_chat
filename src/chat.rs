use conrod;
use std::time::Instant;
pub mod message {
    pub use custom_widget::chatview::Message;
}
pub use conrod_keypad::english;
pub use conrod_keypad::sprite;
#[derive(Clone,Debug)]
pub enum ConrodMessage {
    Event(Instant, conrod::event::Input),
    Thread(Instant),
}
