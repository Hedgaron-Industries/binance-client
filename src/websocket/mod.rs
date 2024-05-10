//!
//! The Binance WebSocket adapter.
//!

pub mod event;

use std::sync::mpsc;
use std::thread;

use websocket::client::ClientBuilder;
use websocket::ws::dataframe::DataFrame;
use websocket::OwnedMessage;

use crate::error::Error;

use self::event::depth::Depth;
use self::event::trade::Trade;
use self::event::Event;

///
/// The Binance WebSocket client.
///
#[derive(Debug, Clone)]
pub struct Client {}

impl Client {
    ///
    /// Runs the `symbol`-dedicated trade and depth streams.
    ///
    /// If `depth` is `None`, it is not requested.
    ///
    pub fn run(symbol: &str, depth_period: Option<u64>) -> Result<mpsc::Receiver<Event>, Error> {
        let (tx, rx) = mpsc::channel();

        Self::subscribe::<Trade>(
            format!(
                "wss://stream.binance.com:9443/ws/{}@trade",
                symbol.to_ascii_lowercase()
            )
            .as_str(),
            tx.clone(),
        )?;

        if let Some(depth_period) = depth_period {
            Self::subscribe::<Depth>(
                format!(
                    "wss://stream.binance.com:9443/ws/{}@depth@{depth_period}ms",
                    symbol.to_ascii_lowercase()
                )
                .as_str(),
                tx.clone(),
            )?;
        }

        Ok(rx)
    }

    ///
    /// Subscribes to a particular stream.
    ///
    fn subscribe<E>(url: &str, tx: mpsc::Sender<Event>) -> Result<(), Error>
    where
        E: Into<Event> + serde::de::DeserializeOwned,
    {
        let mut client = ClientBuilder::new(url)
            .expect("WebSocket address is valid")
            .connect_secure(None)
            .map_err(Error::WebSocket)?;

        thread::spawn(move || loop {
            let message = match client.recv_message() {
                Ok(message) => {
                    if message.is_ping() {
                        log::debug!("Received ping");
                        match client.send_message(&OwnedMessage::Pong(b"pong frame".to_vec())) {
                            Ok(()) => log::debug!("Sent pong"),
                            Err(error) => log::warn!("Pong sending error: {}", error),
                        }
                        continue;
                    }

                    message.take_payload()
                }
                Err(error) => {
                    log::error!("Websocket error: {}", error);
                    return;
                }
            };

            if message.is_empty() {
                continue;
            }

            match serde_json::from_slice::<E>(&message) {
                Ok(event) => match tx.send(event.into()) {
                    Ok(()) => {}
                    Err(_) => break,
                },
                Err(error) => log::warn!("Parsing error: {} ({:?})", error, message),
            }
        });

        Ok(())
    }
}
