#[macro_use]
extern crate conrod;
#[macro_use]
extern crate conrod_derive;
#[macro_use]
pub mod custom_widget;
extern crate futures;
extern crate tokio_core;
extern crate serde_json;
extern crate serde;
extern crate find_folder;
#[macro_use]
extern crate serde_derive;
#[cfg(feature="web_socket")]
extern crate websocket;
#[cfg(feature="hotload")]
pub mod dyapplication;
#[cfg(not(feature="hotload"))]
pub mod staticapplication;
pub mod chat;
pub mod app;
#[cfg(test)] mod tests;