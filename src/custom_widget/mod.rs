#[cfg(feature="keypad")]
pub mod chatview;
pub mod item_history;
#[cfg(feature="keypad")]
pub mod chatview_futures_keypad;
#[cfg(feature="keypad")]
pub use custom_widget::chatview_futures_keypad as chatview_futures;

#[cfg(not(feature="keypad"))]
pub mod chatview_futures;

use conrod_core::image;
#[derive(Debug,Clone)]
pub struct Message {
    pub image_id: Option<image::Id>,
    pub name: String,
    pub text: String,
}
