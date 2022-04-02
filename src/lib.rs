use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Deserializer, Serialize};

/// Todo:
pub mod socket;
pub mod util;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Instrument {
    pub base: Symbol,
    pub quote: Symbol,
    pub kind: InstrumentKind,
}

impl Display for Instrument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("({}_{}, {}", self.base, self.quote, self.kind))
    }
}

impl<S> From<(S, S, InstrumentKind)> for Instrument
where
    S: Into<Symbol>,
{
    fn from((base, quote, kind): (S, S, InstrumentKind)) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind
        }
    }
}

impl Instrument {
    pub fn new<S>(base: S, quote: S, kind: InstrumentKind) -> Self
    where
        S: Into<Symbol>
    {
        Self {
            base: base.into(),
            quote: quote.into(),
            kind,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstrumentKind {
    Spot,
    Future(FutureKind),
}

impl Default for InstrumentKind {
    fn default() -> Self {
        Self::Spot
    }
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            InstrumentKind::Spot => "Spot".to_owned(),
            InstrumentKind::Future(kind) => format!("Future::{}", kind)
        })
    }
}

/// Defines the type of a `Future`. If the `Future` has metadata relating to it's expiry, this is
/// included.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FutureKind {
    Perpetual,
    Expiry,
}

impl Default for FutureKind {
    fn default() -> Self {
        Self::Perpetual
    }
}

impl Display for FutureKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            FutureKind::Perpetual => "Perpetual",
            FutureKind::Expiry => "Expiry",
        })
    }
}

/// Barter new type representing a currency symbol `String` identifier.
///
/// eg/ "btc", "eth", "usdt", etc
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Symbol(pub String);

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        String::deserialize(deserializer).map(Symbol::new)
    }
}

impl<S> From<S> for Symbol where S: Into<String> {
    fn from(input: S) -> Self {
        Self(input.into().to_lowercase())
    }
}

impl Symbol {
    pub fn new<S>(input: S) -> Self where S: Into<Symbol> {
        input.into()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use tracing::info;
    use crate::public::{ExchangeId, MarketStream};
    use crate::public::binance::futures::{BinanceFuturesStream};
    use crate::public::model::{MarketEvent, StreamKind, Subscription};
    use crate::socket::error::SocketError;
    use super::*;

    // Todo: Add subscription validation - it currently fails silently
    // Todo: Maybe OutputIter will become an Option<OutputIter>?
    // Todo: Add proper error enum for BinanceMessage in Barter-Data
    //     '--> eg/ enum BinanceMessage { Error, BinancePayload }
    // Todo: Do I want to keep the name trait Exchange? Do I like the generic ExTransformer, etc.

    #[tokio::test]
    async fn stream_builder_works() -> Result<(), Box<dyn std::error::Error>> {

        let streams = Streams::builder()
            .subscribe(ExchangeId::BinanceFutures, [
                ("btc", "usdt", InstrumentKind::Future, StreamKind::Trades),
                ("eth", "usdt", InstrumentKind::Future, StreamKind::Trades),
            ])
            // .subscribe(ExchangeId::Binance, [
            //     ("btc", "usdt", InstrumentKind::Spot, StreamKind::Trades),
            //     ("eth", "usdt", InstrumentKind::Spot, StreamKind::Trades),
            // ])
            // .subscribe(ExchangeId::Ftx, [
            //     ("btc", "usdt", InstrumentKind::Spot, StreamKind::Trades),
            //     ("eth", "usdt", InstrumentKind::Spot, StreamKind::Trades),
            // ])
            .init()
            .await?;

        // Select individual exchange streams
        // let mut futures_stream = streams
        //     .select(ExchangeId::BinanceFutures)
        //     .ok_or(SocketError::Unidentifiable("".to_owned()))?; // Todo


        // let mut ftx_stream = streams
        //     .select(ExchangeId::Ftx)
        //     .ok_or(SocketError::Unidentifiable("".to_owned()))?; // Todo:

        // Join the remaining exchange streams into one
        let mut joined_stream = streams.join().await;

        while let Some(event) = joined_stream.recv().await {
            println!("{:?}", event);
        }


        Ok(())
    }
}