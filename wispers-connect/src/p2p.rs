//! Peer-to-peer connection types.
//!
//! This module provides the types for establishing and managing P2P connections
//! between activated nodes. Two transport types are supported:
//!
//! - **Datagram**: Raw UDP with AES-GCM encryption. Low overhead, unreliable delivery.
//! - **Stream**: QUIC streams with TLS-PSK. Reliable, ordered delivery with flow control.

use thiserror::Error;

use crate::encryption::{EncryptionError, P2pCipher};
use crate::ice::{IceAnswerer, IceCaller, IceError};
use crate::juice::State as IceState;
use crate::quic::{self, QuicConnection, QuicError, QuicStream};

/// Connection state for a P2P connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// ICE is gathering candidates.
    Gathering,
    /// ICE is connecting to the peer.
    Connecting,
    /// Connection is established and ready for data.
    Connected,
    /// Connection has been disconnected.
    Disconnected,
    /// Connection failed (ICE failure or other error).
    Failed,
}

impl ConnectionState {
    fn from_ice_state(ice_state: IceState) -> Self {
        match ice_state {
            IceState::Disconnected => ConnectionState::Disconnected,
            IceState::Gathering => ConnectionState::Gathering,
            IceState::Connecting => ConnectionState::Connecting,
            IceState::Connected | IceState::Completed => ConnectionState::Connected,
            IceState::Failed | IceState::Unknown(_) => ConnectionState::Failed,
        }
    }

    /// Returns true if the connection is established and ready for data.
    pub fn is_connected(self) -> bool {
        matches!(self, ConnectionState::Connected)
    }

    /// Returns true if the connection is disconnected or failed.
    pub fn is_disconnected(self) -> bool {
        matches!(self, ConnectionState::Disconnected | ConnectionState::Failed)
    }
}

/// Error type for P2P connection operations.
#[derive(Debug, Error)]
pub enum P2pError {
    #[error("hub error: {0}")]
    Hub(#[from] crate::hub::HubError),

    #[error("ICE error: {0}")]
    Ice(#[from] IceError),

    #[error("encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    #[error("QUIC error: {0}")]
    Quic(#[from] QuicError),

    #[error("peer rejected connection: {0}")]
    PeerRejected(String),

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("disconnected")]
    Disconnected,
}

/// A peer-to-peer connection to another node (caller side).
///
/// This provides encrypted UDP communication with a peer node after
/// successful ICE negotiation.
pub struct DatagramConnection {
    /// The peer's node number.
    pub peer_node_number: i32,

    /// Connection ID assigned by the answerer.
    pub connection_id: i64,

    /// The underlying ICE connection.
    ice: IceCaller,

    /// Cipher for encrypting/decrypting packets.
    cipher: P2pCipher,
}

impl DatagramConnection {
    /// Create a new P2P connection (internal use).
    pub(crate) fn new(
        peer_node_number: i32,
        connection_id: i64,
        ice: IceCaller,
        shared_secret: [u8; 32],
    ) -> Result<Self, P2pError> {
        let cipher = P2pCipher::new_caller(&shared_secret, connection_id)?;
        Ok(Self {
            peer_node_number,
            connection_id,
            ice,
            cipher,
        })
    }

    /// Send data to the peer.
    ///
    /// The data is encrypted before transmission.
    pub fn send(&self, data: &[u8]) -> Result<(), P2pError> {
        if self.state().is_disconnected() {
            return Err(P2pError::Disconnected);
        }
        let encrypted = self.cipher.encrypt(data)?;
        self.ice.send(&encrypted)?;
        Ok(())
    }

    /// Receive data from the peer.
    ///
    /// Returns decrypted data from the peer.
    pub async fn recv(&self) -> Result<Vec<u8>, P2pError> {
        if self.state().is_disconnected() {
            return Err(P2pError::Disconnected);
        }
        let encrypted = self.ice.recv().await?;
        let decrypted = self.cipher.decrypt(&encrypted)?;
        Ok(decrypted)
    }

    /// Close the connection.
    pub fn close(self) {
        self.ice.close();
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        ConnectionState::from_ice_state(self.ice.state())
    }

    /// Returns true if the connection is established and ready for data.
    pub fn is_connected(&self) -> bool {
        self.state().is_connected()
    }
}

/// A peer-to-peer connection to another node (answerer side).
pub struct DatagramConnectionAnswerer {
    /// The peer's node number (the caller).
    pub peer_node_number: i32,

    /// Connection ID we assigned.
    pub connection_id: i64,

    /// The underlying ICE connection.
    ice: IceAnswerer,

    /// Cipher for encrypting/decrypting packets.
    cipher: P2pCipher,
}

impl DatagramConnectionAnswerer {
    /// Create a new P2P connection answerer (internal use).
    pub(crate) fn new(
        peer_node_number: i32,
        connection_id: i64,
        ice: IceAnswerer,
        shared_secret: [u8; 32],
    ) -> Result<Self, P2pError> {
        let cipher = P2pCipher::new_answerer(&shared_secret, connection_id)?;
        Ok(Self {
            peer_node_number,
            connection_id,
            ice,
            cipher,
        })
    }

    /// Wait for the ICE connection to complete.
    pub async fn connect(&self) -> Result<(), P2pError> {
        self.ice.connect().await?;
        Ok(())
    }

    /// Send data to the peer.
    pub fn send(&self, data: &[u8]) -> Result<(), P2pError> {
        if self.state().is_disconnected() {
            return Err(P2pError::Disconnected);
        }
        let encrypted = self.cipher.encrypt(data)?;
        self.ice.send(&encrypted)?;
        Ok(())
    }

    /// Receive data from the peer.
    ///
    /// Returns decrypted data from the peer.
    pub async fn recv(&self) -> Result<Vec<u8>, P2pError> {
        if self.state().is_disconnected() {
            return Err(P2pError::Disconnected);
        }
        let encrypted = self.ice.recv().await?;
        let decrypted = self.cipher.decrypt(&encrypted)?;
        Ok(decrypted)
    }

    /// Close the connection.
    pub fn close(self) {
        self.ice.close();
    }

    /// Get the current connection state.
    pub fn state(&self) -> ConnectionState {
        ConnectionState::from_ice_state(self.ice.state())
    }

    /// Returns true if the connection is established and ready for data.
    pub fn is_connected(&self) -> bool {
        self.state().is_connected()
    }
}

//-- Stream connections (QUIC) ---------------------------------------------------------------------

/// A stream-based P2P connection to another node (caller side).
///
/// This provides QUIC streams for reliable, ordered communication with a peer
/// node. Unlike `DatagramConnection`, streams provide flow control and
/// automatic retransmission.
pub struct StreamConnection {
    /// The peer's node number.
    pub peer_node_number: i32,

    /// Connection ID assigned by the answerer.
    pub connection_id: i64,

    /// The underlying QUIC connection.
    quic: QuicConnection<IceCaller>,
}

impl StreamConnection {
    /// Create and establish a new stream connection (internal use).
    ///
    /// This creates the QUIC connection and performs the handshake.
    /// Returns a fully-established connection ready for stream operations.
    pub(crate) async fn connect(
        peer_node_number: i32,
        connection_id: i64,
        ice: IceCaller,
        shared_secret: [u8; 32],
    ) -> Result<Self, P2pError> {
        let psk = quic::derive_psk(&shared_secret);
        let quic = QuicConnection::new_caller(ice, psk, connection_id)?;
        quic.handshake().await?;
        Ok(Self {
            peer_node_number,
            connection_id,
            quic,
        })
    }

    /// Open a new bidirectional stream.
    ///
    /// Returns a stream that can be used for reading and writing data.
    pub async fn open_stream(&self) -> Result<QuicStream<IceCaller>, P2pError> {
        Ok(self.quic.open_stream().await?)
    }

    /// Accept an incoming stream from the peer.
    ///
    /// Waits for the peer to open a new stream and returns it.
    pub async fn accept_stream(&self) -> Result<QuicStream<IceCaller>, P2pError> {
        Ok(self.quic.accept_stream().await?)
    }

    /// Check if the QUIC connection is established.
    pub async fn is_established(&self) -> bool {
        self.quic.is_established().await
    }

    /// Close the connection.
    pub async fn close(self) -> Result<(), P2pError> {
        self.quic.close().await?;
        Ok(())
    }
}

/// Re-export StunTurnConfig from proto.
pub use crate::hub::proto::StunTurnConfig;
