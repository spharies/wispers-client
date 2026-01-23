//! Peer-to-peer connection types.
//!
//! This module provides the types for establishing and managing P2P connections
//! between activated nodes.

use thiserror::Error;

/// Error type for P2P connection operations.
#[derive(Debug, Error)]
pub enum P2pError {
    #[error("hub error: {0}")]
    Hub(#[from] crate::hub::HubError),

    #[error("peer rejected connection: {0}")]
    PeerRejected(String),

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("ICE negotiation failed")]
    IceNegotiationFailed,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("not yet implemented")]
    NotImplemented,
}

/// A peer-to-peer connection to another node.
///
/// This provides encrypted UDP communication with a peer node after
/// successful ICE negotiation.
pub struct P2pConnection {
    /// The peer's node number.
    pub peer_node_number: i32,

    /// Connection ID assigned by the answerer.
    pub connection_id: i64,

    /// Shared secret derived from X25519 key exchange.
    #[allow(dead_code)]
    shared_secret: [u8; 32],
    // TODO Phase 2: Add IceCaller/IceAnswerer for actual data transport
}

impl P2pConnection {
    /// Create a new P2P connection (internal use).
    pub(crate) fn new(
        peer_node_number: i32,
        connection_id: i64,
        shared_secret: [u8; 32],
    ) -> Self {
        Self {
            peer_node_number,
            connection_id,
            shared_secret,
        }
    }

    /// Send data to the peer.
    ///
    /// The data is encrypted using the shared secret before transmission.
    pub async fn send(&self, _data: &[u8]) -> Result<(), P2pError> {
        // TODO Phase 2: Implement with ICE agent
        Err(P2pError::NotImplemented)
    }

    /// Receive data from the peer.
    ///
    /// Returns decrypted data from the peer.
    pub async fn recv(&self) -> Result<Vec<u8>, P2pError> {
        // TODO Phase 2: Implement with ICE agent
        Err(P2pError::NotImplemented)
    }

    /// Close the connection.
    pub fn close(self) {
        // TODO Phase 2: Clean up ICE agent
    }
}

/// Re-export StunTurnConfig from proto.
pub use crate::hub::proto::StunTurnConfig;
