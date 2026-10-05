//! Device redirection ([MS-RDPEFS]), the server's side, served in one of two ways:
//!
//! * [`RdpdrServerFactory`]: `ironrdp-rdpdr`'s server serves the devices the client redirects
//!   through the embedder's [`RdpdrServerBackend`], printers among them when the factory
//!   [serves printers](RdpdrServerFactory::serve_printers). [`crate::ServerEvent::Rdpdr`] sends the
//!   client the I/O requests the backend issues.
//! * [`RdpdrRelayFactory`]: [`RdpdrRelay`] relays the channel. What the client sends on it goes
//!   to the embedder as it arrived, and what the embedder submits
//!   ([`crate::ServerEvent::RdpdrRelay`]) goes to the client unchanged. A proxy relays the channel
//!   between its client and the server it connects to on that client's behalf, so every device
//!   the client redirects reaches that server, served by it.
//!
//! [MS-RDPEFS]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpefs/34d9de58-b2b5-40b6-b970-f82d4603bdb5

use core::fmt;

use ironrdp_core::{Encode, EncodeResult, WriteCursor, ensure_size, impl_as_any};
use ironrdp_pdu::PduResult;
use ironrdp_pdu::gcc::ChannelName;
use ironrdp_rdpdr::server::RdpdrServer;
pub use ironrdp_rdpdr::server::{NoopRdpdrServerBackend, RdpdrServerBackend, RdpdrServerMessage};
use ironrdp_svc::{CompressionCondition, SvcEncode, SvcMessage, SvcProcessor, SvcServerProcessor};

use crate::ServerEventSender;

pub trait RdpdrServerFactory: ServerEventSender {
    fn build_backend(&self) -> Box<dyn RdpdrServerBackend>;

    /// Whether the server lets the client announce its printers
    /// ([`RdpdrServer::with_printers`]). Defaults to `false`.
    fn serve_printers(&self) -> bool {
        false
    }
}

/// The embedder's side of a relayed channel.
pub trait RdpdrRelayHandler: Send + fmt::Debug {
    /// The client sent `message` on the channel.
    fn received(&mut self, message: Vec<u8>);
}

pub trait RdpdrRelayFactory: ServerEventSender + Send {
    fn build_handler(&self) -> Box<dyn RdpdrRelayHandler>;
}

/// The `rdpdr` static channel, relayed.
#[derive(Debug)]
pub struct RdpdrRelay {
    handler: Box<dyn RdpdrRelayHandler>,
}

impl_as_any!(RdpdrRelay);

impl RdpdrRelay {
    pub fn new(handler: Box<dyn RdpdrRelayHandler>) -> Self {
        Self { handler }
    }

    /// The channel message `message`, to send the client.
    pub fn relay(message: Vec<u8>) -> Vec<SvcMessage> {
        vec![SvcMessage::from(RelayedMessage(message))]
    }
}

impl SvcProcessor for RdpdrRelay {
    fn channel_name(&self) -> ChannelName {
        RdpdrServer::NAME
    }

    fn compression_condition(&self) -> CompressionCondition {
        CompressionCondition::WhenRdpDataIsCompressed
    }

    /// The server on the far side announces itself, not this one: the relay sends nothing until
    /// the embedder submits it.
    fn start(&mut self) -> PduResult<Vec<SvcMessage>> {
        Ok(Vec::new())
    }

    fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
        self.handler.received(payload.to_vec());
        Ok(Vec::new())
    }
}

impl SvcServerProcessor for RdpdrRelay {}

/// A channel message relayed as it is.
struct RelayedMessage(Vec<u8>);

impl Encode for RelayedMessage {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
        ensure_size!(in: dst, size: self.0.len());
        dst.write_slice(&self.0);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "RelayedRdpdrMessage"
    }

    fn size(&self) -> usize {
        self.0.len()
    }
}

impl SvcEncode for RelayedMessage {}
