use crate::services::event_bus::{EventBus, Request};

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use reqwasm::websocket::{futures::WebSocket, Message};
use wasm_bindgen_futures::spawn_local;
use yew_agent::Dispatched;

pub struct WebsocketService {
    pub tx: Sender<String>,
}

impl WebsocketService {
    pub fn new() -> Self {
        let ws = WebSocket::open("ws://[::1]:8080").expect("Failed to connect to WebSocket server");
        log::info!("WebSocket connection established");

        let (mut write, mut read) = ws.split();
        let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<String>(1000);

        let mut event_bus = EventBus::dispatcher();

        spawn_local(async move {
            while let Some(s) = in_rx.next().await {
                log::debug!("got event from channel! {}", s);
                write.send(Message::Text(s)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        log::debug!("from websocket: {}", data);
                        event_bus.send(Request::EventBusMsg(data));
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(val) = decoded {
                            log::debug!("from websocket: {}", val);
                            event_bus.send(Request::EventBusMsg(val.into()));
                        }
                    }
                    Err(e) => {
                        log::error!("ws: {:?}", e)
                    }
                }
            }
            log::debug!("WebSocket Closed");
        });

        Self { tx: in_tx }
    }
}
