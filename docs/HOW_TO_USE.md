# How to use it

This guide shows how to integrate Wispers Connect into your application. 

## Integration patterns

There are two main approaches:

* **Embed the library** — This gives you full control over the node lifecycle,
  serving, and peer-to-peer connections, and it lets you define the protocol on
  top of those peer-to-peer connections. This is the right choice if you're
  building your own software. The library is written in Rust and exposes a C
  FFI. Wrappers exist for Kotlin/Android and Go; more are planned (Swift,
  Python). See [Building](../README.md#building) for setup instructions.

* **Use wconnect as a sidecar** — The `wconnect` tool already implements some of
  the most popular and generic use cases of Wispers Connect — port forwarding
  and HTTP/SOCKS proxying. Run it as a sidecar process if you have an existing
  web app or TCP service and just want to make it reachable across devices.

## Configuring the Wispers Connect backends

Before the library and wconnect can do their work, you need to tell Wispers
about your use case.

### The Wispers Connect web app

To get started, you need at least one domain and an API key for it.

1. Get an account on https://connect.wispers.dev.
2. Choose a **domain**. You can think of domains as corresponding to use
   cases. For personal use and experimental projects, the automatically created
   "Default" domain is sufficient. If you're planning a new application based on
   Wispers Connect, you should probably give it its own domain.
3. Create at least one **API key** and note it down. You can always create new
   keys, but be careful with revoking them — if a production service relies on
   an API key, revoking it can cause the service to fail nearly instantly. The
   CLI tools accept API keys either as a CLI argument or as the environment
   variable `WC_API_KEY`.

### The REST API

Once you have an API key it's time to talk to the REST API, or use the `wcadm`
tool to do it for you.

TODO: Explain the concrete methods to deal with connectivity groups. Also check
that the API documentation on the server is sufficient

## Using the library

### Storage

A Wispers Connect node has very little state, but that state should get stored
securely. The library only comes with two built-in options, in-memory for
testing, and file-based for CLI tools. For everything else, you need to provide
your own implementation — either by implementing the `NodeStateStore` trait in
Rust, or by implementing the equivalent FFI storage callbacks from a wrapper
language. If possible, you'll want to use your platform's secure storage, like
for example the macOS Keychain.

The Kotlin wrapper implementation contains an example: See
`/wrappers/kotlin/src/main/kotlin/dev/wispers/connect/storage`

### Node lifecycle

The main object you'll deal with is the `Node`. It can be in various lifecycle
states: "pending", "registered", "activated". The typical flow to get a Node up
and running is this:

1. Instantiate a `NodeStorage` object using your storage implementation, then
   call `restore_or_init_node()` on it. This will read the state from storage
   (or if that's empty, initialise it as "pending") and return a Node.
2. Get the Node into the "activated" state.
   * If the Node is "pending", get a registration token and call
     `node.register(token)`
   * If the Node is "registered", get an activation code and call
     `node.activate(code)`
3. Once the node is activated (check `node.state()`), it's fully functional. You
   can
   * `start_serving()` to wait for other nodes to open connections to this one
   * `connect_quic()` or `connect_udp()` to open a peer-to-peer connection to
     another node
   * Query `group_info()` to get the state of all nodes in the connectivity
     group

If you need to reset a node, you can also call `logout()`. This will revoke the
node's entry from the roster and deregister the node from the hub.

To understand what the different node states really mean, check out the
explanation in [HOW_IT_WORKS.md](HOW_IT_WORKS.md).

### Serving

TODO: what serving does, how do handle incoming connections

### Opening connections

TODO: How to open _and use_ UDP and QUIC connections

### Error handling

<!-- TODO: cover common error scenarios:
     - Hub unreachable
     - Unauthenticated (node removed server-side)
     - Invalid activation code
     - Peer rejected / unavailable
     - State-inappropriate operations (InvalidState)
-->

### Examples

Complete, runnable examples for each wrapper live in the `examples/` directory.

<!-- TODO: create examples/ with at minimum:
     - examples/rust/    — simple echo service
     - examples/go/      — same in Go
     - examples/kotlin/  — Android integration
     Each should be self-contained and buildable. -->

## Using wconnect as a sidecar

<!-- TODO: explain the sidecar pattern:
     - wconnect runs as a separate process alongside your app
     - Your app doesn't link the library at all
     - wconnect handles registration, activation, serving

     ### Port forwarding
     - Forward a local TCP port to a peer node's port
     - Example: expose a dev server to a teammate's laptop

     ### HTTP proxying
     - Proxy HTTP requests to a peer node
     - Example: access an internal web app from outside the office

     ### Running as a daemon
     - `wconnect serve -d` for background operation
     - Status, shutdown via Unix socket
     - See INTERNALS.md for daemon architecture details
-->

## Real-world examples

These show how the pieces fit together in actual deployments.

### Wispers Files (library integration)

<!-- TODO: describe the Files architecture:
     - Desktop app (Tauri) and Android app both embed the library
     - Registration via files.wispers.dev web UI + deep links
     - Serving runs in the background for file sync
     - QUIC streams for reliable file transfer
     - Point to the Files source as a reference -->

### Internal web app (wconnect sidecar)

<!-- TODO: describe a concrete scenario:
     - A team runs an internal web app (e.g. wiki, dashboard)
     - One team member runs `wconnect serve` + `wconnect proxy` on the server
     - Other team members run `wconnect` on their laptops
     - The web app is now accessible across NATs without a VPN
     - No code changes to the web app needed -->
