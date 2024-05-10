//!
//! The merged data, received via WebSocket.
//!

pub mod depth;
pub mod trade;

use self::depth::Depth;
use self::trade::Trade;

///
/// The merged data, received via WebSocket.
///
#[derive(Debug, Clone)]
pub enum Event {
    /// The trade event from the `trade` stream.
    Trade(Trade),
    /// The depth event from the `depth` stream.
    Depth(Depth),
}

impl From<Trade> for Event {
    fn from(inner: Trade) -> Self {
        Self::Trade(inner)
    }
}

impl From<Depth> for Event {
    fn from(inner: Depth) -> Self {
        Self::Depth(inner)
    }
}
