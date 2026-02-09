//! HTTP proxy for accessing web servers on remote nodes.
//!
//! This module implements a forward HTTP proxy that allows browsers/clients
//! to access web servers running on nodes in the connectivity group using
//! hostnames like `http://3.wispers.link/`.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use wispers_connect::{Node, NodeState};

/// Run the HTTP proxy server.
pub async fn run(hub_override: Option<&str>, profile: &str, bind_addr: &str) -> Result<()> {
    let storage = super::get_storage(hub_override, profile)?;
    let node = storage
        .restore_or_init_node()
        .await
        .context("failed to load node state")?;

    if node.state() != NodeState::Activated {
        anyhow::bail!(
            "Node must be activated to use HTTP proxy. Current state: {:?}",
            node.state()
        );
    }

    let listener = TcpListener::bind(bind_addr)
        .await
        .with_context(|| format!("failed to bind to {}", bind_addr))?;

    println!("HTTP proxy listening on {}", bind_addr);
    println!("Configure your browser/client to use this as HTTP proxy");
    println!("Example: curl --proxy http://{} http://3.wispers.link/", bind_addr);

    let node = Arc::new(node);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("Accepted connection from {}", addr);
                let node = Arc::clone(&node);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, node).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}

/// Handle a single client connection.
async fn handle_connection(stream: TcpStream, _node: Arc<Node>) -> Result<()> {
    let peer = stream.peer_addr()?;

    // TODO: Phase 2 - Parse HTTP request
    // TODO: Phase 3 - Get/create QUIC connection from pool
    // TODO: Phase 4 - Forward request to target node
    // TODO: Phase 5 - Handle keep-alive

    // For now, just close the connection with a placeholder response
    use tokio::io::AsyncWriteExt;
    let mut stream = stream;
    stream.write_all(b"HTTP/1.1 501 Not Implemented\r\n\r\nProxy not yet implemented\n").await?;

    println!("Connection from {} closed (not yet implemented)", peer);
    Ok(())
}
