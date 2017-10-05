#[cfg(feature="web_socket")]
extern crate websocket;
extern crate futures;
extern crate tokio_core;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::sink::Sink;
use futures::sync::mpsc;
use futures::stream::Stream;
use websocket::ClientBuilder;
use websocket::message::OwnedMessage;
use websocket::Message;
const CONNECTION: &'static str = "ws://www.google.com";
#[test]
fn websocket_ping() {
    let mut core = Core::new().unwrap();
    //  let (gui_tx,gui_rx) = std::sync::mpsc::channel();
    let (tx, rx) = mpsc::channel::<OwnedMessage>(1);
    let runner = ClientBuilder::new(CONNECTION)
            .unwrap()
           // .add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let content = match msg {
                        OwnedMessage::Close(e) => Some(Message::from(OwnedMessage::Close(e))),
                        OwnedMessage::Ping(d) => Some(Message::from(OwnedMessage::Ping(d))),
                        _ => None,
                    };
                    // ... and send that string _to_ the GUI.
                    println!("content {:?}",content);
                    Ok(())
                });
                let writer = rx
            .map_err(|()| unreachable!("rx can't fail"))
            .fold(to_server, |to_server, msg| {
                let h= msg.clone();
                 to_server.send(OwnedMessage::from(h))
            })
            .map(|_| ());
                // Use select to allow either the reading or writing half dropping to drop the other
                // half. The `map` and `map_err` here effectively force this drop.
                reader.select(writer).map(|_| ()).map_err(|(err, _)| err)
            });
    core.run(runner).unwrap();
}
