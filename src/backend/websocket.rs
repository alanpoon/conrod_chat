pub mod client {
    use std;
    use std::thread;
    use std::io::stdin;
    use tokio_core::reactor::Core;
    use futures::future::Future;
    use futures::sink::Sink;
    use futures::stream::Stream;
    use futures::sync::mpsc;
    use websocket::result::WebSocketError;
    use websocket::{ClientBuilder, OwnedMessage, Message};
    pub fn run<'a>(con: &'static str,
                   gui: std::sync::mpsc::Sender<Message<'a>>,
                   rx: mpsc::Receiver<Message<'a>>) {
        let mut core = Core::new().unwrap();
        let runner = ClientBuilder::new(con)
            .unwrap()
            .add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let content = match msg {
                        OwnedMessage::Close(e) => Some(Message::from(OwnedMessage::Close(e))),
                        OwnedMessage::Ping(d) => Some(Message::from(OwnedMessage::Ping(d))),
                        OwnedMessage::Text(f) => {
                            gui.send(Message::text(f)).unwrap();
                            None
                        }
                        _ => None,
                    };
                    // ... and send that string _to_ the GUI.

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

}
