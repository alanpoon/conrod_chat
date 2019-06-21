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
    use serde_json;
    #[derive(Serialize,Deserialize)]
    #[serde(tag = "connection_status", content = "c")]
    pub enum ConnectionStatus {
        Error(ConnectionError),
        Ok,
    }
    #[derive(Serialize,Deserialize)]
    pub enum ConnectionError {
        NotConnectedToInternet,
        CannotFindServer,
        InvalidDestination,
    }

    pub fn run<'a>(con: String,
                   gui: std::sync::mpsc::Sender<Message<'a>>,
                   rx: mpsc::Receiver<Message<'a>>)
                   -> Result<(), ConnectionError> {
        println!("run");
        let gui_c = gui.clone();
        match ClientBuilder::new(&con) {
            Ok(c) => {
                let mut core = Core::new().unwrap();
                let runner= c.add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let _content = match msg {
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
                        let g = json!({
                                          "connection_status": ConnectionStatus::Ok
                                      });
                        gui.clone().send(Message::text(g.to_string())).unwrap();
                        Ok(())
                    }
                    Err(_er) => {
                        let g = json!({
                                          "connection_status":
                                          ConnectionStatus::Error(ConnectionError::CannotFindServer)
                                      });
                        gui.clone().send(Message::text(g.to_string())).unwrap();
                        Err(ConnectionError::CannotFindServer)
                    }
                }
            }
            Err(er) => {
                gui.clone().send(Message::text(er.clone().description().to_owned())).unwrap();
                Err(ConnectionError::InvalidDestination)
            }
        }

    }

    pub fn run_owned_message(con: String,
                             gui: std::sync::mpsc::Sender<OwnedMessage>,
                             rx: mpsc::Receiver<OwnedMessage>)
                             -> Result<(), ConnectionError> {
        println!("run");
        let gui_c = gui.clone();
        match ClientBuilder::new(&con) {
            Ok(_) => {
                let mut core = Core::new().unwrap();
                let runner = ClientBuilder::new(&con)
            .unwrap()
            .add_protocol("rust-websocket")
            .async_connect_insecure(&core.handle())
            .and_then(move |(duplex, _)| {
                let (to_server, from_server) = duplex.split();
                let reader = from_server.for_each(move |msg| {
                    // ... convert it to a string for display in the GUI...
                    let _content = match msg {
                        OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
                        OwnedMessage::Ping(d) => Some(OwnedMessage::Ping(d)),
                        OwnedMessage::Text(f) => {
                            gui_c.send(OwnedMessage::Text(f)).unwrap();
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
                match core.run(runner) {
                    Ok(_) => {
                        println!("connected");
                        let g = serde_json::to_string(&ConnectionStatus::Ok).unwrap();
                        gui.clone().send(OwnedMessage::Text(g)).unwrap();
                        Ok(())
                    }
                    Err(_er) => {
                        let g = serde_json::to_string(&ConnectionStatus::Error(ConnectionError::CannotFindServer)).unwrap();
                        gui.clone().send(OwnedMessage::Text(g)).unwrap();
                        Err(ConnectionError::CannotFindServer)
                    }
                }
            }
            Err(er) => {
                gui.clone().send(OwnedMessage::Text(er.clone().description().to_owned())).unwrap();
                Err(ConnectionError::InvalidDestination)
            }
        }

    }
}
