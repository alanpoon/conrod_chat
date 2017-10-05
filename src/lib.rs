#[macro_use]
extern crate conrod;
#[macro_use]
extern crate conrod_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
pub mod custom_widget;
extern crate futures;
extern crate tokio_core;
#[cfg(feature="web_socket")]
extern crate websocket;
#[cfg(feature="hotload")]
extern crate libloading;
#[cfg(feature="hotload")]
pub mod dyapplication;
#[cfg(not(feature="hotload"))]
pub mod staticapplication;
pub mod chat;
pub mod app;
pub mod backend;
