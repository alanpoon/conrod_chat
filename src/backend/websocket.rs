pub mod client {
    use std;
    use tokio_core::reactor::Core;
    use futures::future::Future;
    use futures::sink::Sink;
    use futures::stream::Stream;
    use futures::sync::mpsc;
    use websocket::ClientBuilder;
    pub use websocket::OwnedMessage;
    pub use websocket::Message;
    use std::error::Error;
    pub trait ProcessSender {
        fn process(&self, String);
    }
    pub fn run<'a>(con: &'static str,
                   gui: std::sync::mpsc::Sender<Message<'a>>,
                   rx: mpsc::Receiver<Message<'a>>)
                   -> Result<(), String> {
        println!("run");
        let gui_c = gui.clone();
        match ClientBuilder::new(con) {
            Ok(c) => {
                let mut core = Core::new().unwrap();
                let runner= c.add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let content = match msg {
                        OwnedMessage::Close(e) => Some(Message::from(OwnedMessage::Close(e))),
                        OwnedMessage::Ping(d) => Some(Message::from(OwnedMessage::Ping(d))),
                        OwnedMessage::Text(f) => {
                            gui_c.clone().send(Message::text(f)).unwrap();
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
                match core.run(runner) {
                    Ok(_) => {
                        println!("connected");
                        gui.clone().send(Message::text("connected")).unwrap();
                        Ok(())
                    }
                    Err(_er) => {
                        println!("{:?}", _er);
                        let f = format!("{}", _er);
                        gui.clone().send(Message::text(f.clone())).unwrap();
                        Err(f)
                    }
                }
            }
            Err(er) => {
                gui.clone().send(Message::text(er.clone().description().to_owned())).unwrap();
                Err(er.description().to_owned())
            }
        }

    }
    pub fn run_with_trait<G: ProcessSender + Clone + Sink, T: Clone>(con: &'static str,
                                                                     gui: G,
                                                                     rx: mpsc::Receiver<T>)
        where OwnedMessage: std::convert::From<T>
    {
        println!("run");
        let gui_c = gui.clone();
        match ClientBuilder::new(con) {
            Ok(c) => {
                let mut core = Core::new().unwrap();
                let runner= c.add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                     let content = match msg {
                        OwnedMessage::Close(e) => Some(Message::from(OwnedMessage::Close(e))),
                        OwnedMessage::Ping(d) => Some(Message::from(OwnedMessage::Ping(d))),
                        OwnedMessage::Text(f) => {
                            gui_c.clone().process(f);
                            None
                        }
                        _ => None,
                    };
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
                match core.run(runner) {
                    Ok(_) => {
                        println!("connected");
                        gui.clone().process("connected".to_owned());
                    }
                    Err(_er) => {
                        println!("{:?}", _er);
                        let f = format!("{}", _er);
                        gui.clone().process(f);
                    }
                }
            }
            Err(er) => {
                gui.clone().process(er.description().to_owned());
            }
        }

    }
    pub fn run_owned_message(con: &'static str,
                             gui: std::sync::mpsc::Sender<OwnedMessage>,
                             rx: mpsc::Receiver<OwnedMessage>) {
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
                        OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                        OwnedMessage::Ping(d) => Some(OwnedMessage::Ping(d)),
                        OwnedMessage::Text(f) => {
                            gui.send(OwnedMessage::Text(f)).unwrap();
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
                 to_server.send(h)
            })
            .map(|_| ());

                // Use select to allow either the reading or writing half dropping to drop the other
                // half. The `map` and `map_err` here effectively force this drop.
                reader.select(writer).map(|_| ()).map_err(|(err, _)| err)
            });
        core.run(runner).unwrap();
    }
}
