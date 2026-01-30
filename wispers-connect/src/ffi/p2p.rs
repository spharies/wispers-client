//! FFI bindings for P2P connections.

use super::callbacks::CallbackContext;
use super::handles::ActivatedImpl;
use super::runtime;
use crate::errors::WispersStatus;
use crate::p2p::UdpConnection;
use std::ffi::c_void;
use std::os::raw::c_int;

// Helper to send raw pointers across threads
struct SendablePtr(*mut WispersUdpConnectionHandle);
unsafe impl Send for SendablePtr {}

impl SendablePtr {
    /// Get a reference to the inner UdpConnection.
    /// SAFETY: The caller must ensure the pointer is valid.
    unsafe fn get(&self) -> &UdpConnection {
        unsafe { &(*self.0).0 }
    }
}

/// Opaque handle to a UDP P2P connection.
pub struct WispersUdpConnectionHandle(pub(crate) UdpConnection);

/// Callback that receives a UDP connection handle.
pub type WispersUdpConnectionCallback = Option<
    unsafe extern "C" fn(
        ctx: *mut c_void,
        status: WispersStatus,
        connection: *mut WispersUdpConnectionHandle,
    ),
>;

/// Callback that receives received data.
pub type WispersDataCallback = Option<
    unsafe extern "C" fn(
        ctx: *mut c_void,
        status: WispersStatus,
        data: *const u8,
        len: usize,
    ),
>;

// Free function

#[unsafe(no_mangle)]
pub extern "C" fn wispers_udp_connection_free(handle: *mut WispersUdpConnectionHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

/// Connect to a peer node using UDP transport.
///
/// The activated handle is NOT consumed.
/// On success, callback receives the UDP connection handle.
#[unsafe(no_mangle)]
pub extern "C" fn wispers_activated_node_connect_udp_async(
    handle: *mut super::handles::WispersActivatedNodeHandle,
    peer_node_number: c_int,
    ctx: *mut c_void,
    callback: WispersUdpConnectionCallback,
) -> WispersStatus {
    if handle.is_null() {
        return WispersStatus::NullPointer;
    }

    let callback = match callback {
        Some(cb) => cb,
        None => return WispersStatus::MissingCallback,
    };

    let wrapper = unsafe { &*handle };
    let ctx = CallbackContext(ctx);

    // Extract what we need before spawning
    let (hub_addr, registration, signing_key, x25519_key, roster) = match &wrapper.0 {
        ActivatedImpl::InMemory(activated) => (
            activated.hub_addr(),
            activated.registration().clone(),
            activated.signing_key().clone(),
            activated.x25519_key().clone(),
            activated.roster().clone(),
        ),
        ActivatedImpl::Foreign(activated) => (
            activated.hub_addr(),
            activated.registration().clone(),
            activated.signing_key().clone(),
            activated.x25519_key().clone(),
            activated.roster().clone(),
        ),
    };

    runtime::spawn(async move {
        let result = connect_udp_impl(
            &hub_addr,
            &registration,
            &signing_key,
            &x25519_key,
            &roster,
            peer_node_number,
        )
        .await;

        match result {
            Ok(conn) => {
                let h = Box::into_raw(Box::new(WispersUdpConnectionHandle(conn)));
                unsafe {
                    callback(ctx.ptr(), WispersStatus::Success, h);
                }
            }
            Err(status) => {
                unsafe {
                    callback(ctx.ptr(), status, std::ptr::null_mut());
                }
            }
        }
    });

    WispersStatus::Success
}

/// Send data over a UDP connection.
///
/// This is a synchronous, non-blocking operation.
/// The connection handle is NOT consumed.
#[unsafe(no_mangle)]
pub extern "C" fn wispers_udp_connection_send(
    handle: *mut WispersUdpConnectionHandle,
    data: *const u8,
    len: usize,
) -> WispersStatus {
    if handle.is_null() || data.is_null() {
        return WispersStatus::NullPointer;
    }

    let wrapper = unsafe { &*handle };
    let data_slice = unsafe { std::slice::from_raw_parts(data, len) };

    match wrapper.0.send(data_slice) {
        Ok(()) => WispersStatus::Success,
        Err(_) => WispersStatus::ConnectionFailed,
    }
}

/// Receive data from a UDP connection.
///
/// The connection handle is NOT consumed.
/// On success, callback receives the data buffer. The buffer is only valid
/// during the callback invocation.
#[unsafe(no_mangle)]
pub extern "C" fn wispers_udp_connection_recv_async(
    handle: *mut WispersUdpConnectionHandle,
    ctx: *mut c_void,
    callback: WispersDataCallback,
) -> WispersStatus {
    if handle.is_null() {
        return WispersStatus::NullPointer;
    }

    let callback = match callback {
        Some(cb) => cb,
        None => return WispersStatus::MissingCallback,
    };

    // We need to keep the connection alive while receiving.
    // recv() takes &self, so we borrow via raw pointer. This is safe
    // as long as the caller doesn't free the handle while recv is pending.
    // A safer approach would be to wrap in Arc, but that changes the API.
    let ctx = CallbackContext(ctx);
    let conn_ptr_wrapper = SendablePtr(handle);

    runtime::spawn(async move {
        let conn = unsafe { conn_ptr_wrapper.get() };
        let result = conn.recv().await;

        match result {
            Ok(data) => {
                unsafe {
                    callback(ctx.ptr(), WispersStatus::Success, data.as_ptr(), data.len());
                }
            }
            Err(_) => {
                unsafe {
                    callback(ctx.ptr(), WispersStatus::ConnectionFailed, std::ptr::null(), 0);
                }
            }
        }
    });

    WispersStatus::Success
}

/// Close a UDP connection.
///
/// The connection handle is CONSUMED by this call.
#[unsafe(no_mangle)]
pub extern "C" fn wispers_udp_connection_close(handle: *mut WispersUdpConnectionHandle) {
    if handle.is_null() {
        return;
    }

    let wrapper = unsafe { Box::from_raw(handle) };
    wrapper.0.close();
}

// Implementation helper for connect_udp
async fn connect_udp_impl(
    hub_addr: &str,
    registration: &crate::types::NodeRegistration,
    signing_key: &crate::crypto::SigningKeyPair,
    x25519_key: &crate::crypto::X25519KeyPair,
    roster: &crate::hub::proto::roster::Roster,
    peer_node_number: i32,
) -> Result<UdpConnection, WispersStatus> {
    use crate::hub::proto;
    use crate::hub::HubClient;
    use crate::ice::IceCaller;
    use crate::p2p::UdpConnection;
    use ed25519_dalek::pkcs8::DecodePublicKey;
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    // Connect to hub
    let mut client = HubClient::connect(hub_addr)
        .await
        .map_err(|_| WispersStatus::HubError)?;

    // Get STUN/TURN configuration
    let stun_turn_config = client
        .get_stun_turn_config(registration)
        .await
        .map_err(|_| WispersStatus::HubError)?;

    // Create ICE caller and gather candidates
    let ice_caller =
        IceCaller::new(&stun_turn_config).map_err(|_| WispersStatus::ConnectionFailed)?;
    let caller_sdp = ice_caller.local_description().to_string();

    // Build the StartConnectionRequest
    // Sign: answerer_node_number || caller_x25519_public_key || caller_sdp
    let mut message_to_sign = Vec::new();
    message_to_sign.extend_from_slice(&peer_node_number.to_le_bytes());
    message_to_sign.extend_from_slice(&x25519_key.public_key());
    message_to_sign.extend_from_slice(caller_sdp.as_bytes());
    let signature = signing_key.sign(&message_to_sign);

    let request = proto::StartConnectionRequest {
        answerer_node_number: peer_node_number,
        caller_x25519_public_key: x25519_key.public_key().to_vec(),
        caller_sdp,
        signature,
        stun_turn_config: Some(stun_turn_config),
        transport: proto::Transport::Datagram.into(),
    };

    // Send to hub, which forwards to the answerer
    let response = client
        .start_connection(registration, request)
        .await
        .map_err(|_| WispersStatus::HubError)?;

    // Verify answerer's signature against roster
    let peer_node = roster
        .nodes
        .iter()
        .find(|n| n.node_number == peer_node_number)
        .ok_or(WispersStatus::ConnectionFailed)?;

    let verifying_key = VerifyingKey::from_public_key_der(&peer_node.public_key_spki)
        .map_err(|_| WispersStatus::ConnectionFailed)?;

    // Verify signature over: connection_id || answerer_x25519_public_key || answerer_sdp
    let mut message_to_verify = Vec::new();
    message_to_verify.extend_from_slice(&response.connection_id.to_le_bytes());
    message_to_verify.extend_from_slice(&response.answerer_x25519_public_key);
    message_to_verify.extend_from_slice(response.answerer_sdp.as_bytes());

    let sig_bytes: [u8; 64] = response
        .signature
        .clone()
        .try_into()
        .map_err(|_| WispersStatus::ConnectionFailed)?;
    let sig = Signature::from_bytes(&sig_bytes);

    verifying_key
        .verify(&message_to_verify, &sig)
        .map_err(|_| WispersStatus::ConnectionFailed)?;

    // Extract peer's X25519 public key
    let peer_x25519_public: [u8; 32] = response
        .answerer_x25519_public_key
        .try_into()
        .map_err(|_| WispersStatus::ConnectionFailed)?;

    // Derive shared secret
    let shared_secret = x25519_key.diffie_hellman(&peer_x25519_public);

    // Complete ICE connection with answerer's SDP
    ice_caller
        .connect(&response.answerer_sdp)
        .await
        .map_err(|_| WispersStatus::ConnectionFailed)?;

    UdpConnection::new_caller(peer_node_number, response.connection_id, ice_caller, shared_secret)
        .map_err(|_| WispersStatus::ConnectionFailed)
}
