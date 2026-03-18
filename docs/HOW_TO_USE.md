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
tool to do it for you. At this level, we're managing **connectivity groups** and
**node registration** (see [HOW_IT_WORKS.md](HOW_IT_WORKS.md) for a deeper
explanation of these concepts.

We don't go into details about `wcadm` in this doc because `wcadm help` is
clear enough and stays up-to-date.

There's a JSON description of the API at https://connect.wispers.dev/api that
should always be up to date with the latest details. Authentication is done with
your API as bearer token. Here's an overview of the most important bits of the
API:

#### Connectivity group management

* `GET /connectivity-groups` — List connectivity groups (limited to the domain
  matching the API key).
* `POST /connectivity-groups` — Create a connectivity group.\
  Parameters:
  * `name` — Optional, display name of the group
  * `associationKey` — Optional string, e.g. a user ID. Setting this prevents
	creating two groups with the same associationKey. Use this if you're
	associating an ID your side with the connectivity group.
* `DELETE /connectivity-groups/:id` — Delete a connectivity group and all its
  registered nodes.
* `POST /connectivity-groups/:id/reset` — Reset a connectivity group (remove all
  nodes, clear the roster).


#### Node management

* `GET /connectivity-groups/:id` — Get connectivity group state, including its
  nodes.
* `POST /connectivity-groups/:id/registration-tokens` — Create a registration
  token for a new node.\
  Parameters:
  * `nodeName` — Optional display name for the node.
  * `nodeMetadata` — Optional metadata (max 256 chars), in a format defined by
	yourself. JSON blobs work well here (e.g. `{"nodeType": "mobile"}`).
* `PATCH /connectivity-groups/:id/nodes/:nodeNumber` — Update a node.\
  Parameters:
  * `name` — New display name.

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

For a node to be reachable it has to be "serving". That is, it has to be
connected to the hub and wait for connection requests. Call
`node.start_serving()` to kick this off. You get back three objects:

* The `ServingSession` is the runner. Note that you _have to run it_ for things
  to work, for example with `tokio::spawn(session.run())` if you're using the
  Rust interface directly.
* The `ServingHandle` lets you talk to the running serving session (`status()`,
  `generate_activation_code()`, `shutdown()`).
* The `IncomingConnections` object lets you handle connections from peer nodes.
  The Rust interface provides the channels `udp` and `quic` for this purpose,
  giving you `UdpConnection` and `QuicConnection` objects, respectively.

What to do with incoming connections is entirely up to you. You can design any
protocol you like.

### Opening connections

To connect to another node, call `node.connect_quic(target_node)` or
`node.connect_udp(target_node)`, depending on what you need. Most people will
want the QUIC variant.

Once a connection was established, it's your turn to run your app-specific
protocol. Some options:

* A good old line-oriented protocol like SMTP or FTP.
* Varint length prefix, followed by protobuf requests. You get all the nice
  forward- and backward compatibility of protocol buffers, but wire format
  protocol buffers are harder to debug.
* Send and receive JSON objects. This lies somewhere between the first two
  options.

### Error handling

Sometimes, things will go awry. Here's how to deal with some common scenarios:

* Hub unreachable   * `nodeName` — Optional display name for the node.
  * `nodeMetadata` — Optional metadata (max 256 chars), in a format defined by
	yourself. JSON blobs work well here (e.g. `{"nodeType": "mobile"}`).
* `PATCH /connectivity-groups/:id/nodes/:nodeNumber` — Update a node.\
  Parameters:
  * `name` — New display name.

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

For a node to be reachable it has to be "serving". That is, it has to be
connected to the hub and wait for connection requests. Call
`node.start_serving()` to kick this off. You get back three objects:

* The `ServingSession` is the runner. Note that you _have to run it_ for things
  to work, for example with `tokio::spawn(session.run())` if you're using the
  Rust interface directly.
* The `ServingHandle` lets you talk to the running serving session (`status()`,
  `generate_activation_code()`, `shutdown()`).
* The `IncomingConnections` object lets you handle connections from peer nodes.
  The Rust interface provides the channels `udp` and `quic` for this purpose,
  giving you `UdpConnection` and `QuicConnection` objects, respectively.

What to do with incoming connections is entirely up to you. You can design any
protocol you like.

### Opening connections

To connect to another node, call `node.connect_quic(target_node)` or
`node.connect_udp(target_node)`, depending on what you need. Most people will
want the QUIC variant.

Once a connection was established, it's your turn to run your app-specific
protocol. Some options:

* A good old line-oriented protocol like SMTP or FTP.
* Varint length prefix, followed by protobuf requests. You get all the nice
  forward- and backward compatibility of protocol buffers, but wire format
  protocol buffers are harder to debug.
* Send and receive JSON objects. This lies somewhere between the first two
  options.

### Error handling

Sometimes, things will go awry. Here's how to deal with some common scenarios:

* Hub unr  * `nodeName` — Optional display name for the node.
  * `nodeMetadata` — Optional metadata (max 256 chars), in a format defined by
	yourself. JSON blobs work well here (e.g. `{"nodeType": "mobile"}`).
* `PATCH /connectivity-groups/:id/nodes/:nodeNumber` — Update a node.\
  Parameters:
  * `name` — New display name.

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

For a node to be reachable it has to be "serving". That is, it has to be
connected to the hub and wait for connection requests. Call
`node.start_serving()` to kick this off. You get back three objects:

* The `ServingSession` is the runner. Note that you _have to run it_ for things
  to work, for example with `tokio::spawn(session.run())` if you're using the
  Rust interface directly.
* The `ServingHandle` lets you talk to the running serving session (`status()`,
  `generate_activation_code()`, `shutdown()`).
* The `IncomingConnections` object lets you handle connections from peer nodes.
  The Rust interface provides the channels `udp` and `quic` for this purpose,
  giving you `UdpConnection` and `QuicConnection` objects, respectively.

What to do with incoming connections is entirely up to you. You can design any
protocol you like.

### Opening connections

To connect to another node, call `node.connect_quic(target_node)` or
`node.connect_udp(target_node)`, depending on what you need. Most people will
want the QUIC variant.

Once a connection was established, it's your turn to run your app-specific
protocol. Some options:

* A good old line-oriented protocol like SMTP or FTP.
* Varint length prefix, followed by protobuf requests. You get all the nice
  forward- and backward compatibility of protocol buffers, but wire format
  protocol buffers are harder to debug.
* Send and receive JSON objects. This lies somewhere between the first two
  options.

### Error handling

Sometimes, things will go awry. Here's how to deal with some common scenarios:

* State-inappropriate operations (InvalidState) — This happens if you use Node
  methods that don't match the node's current state. For example, if your node
  is "pending" or "registered", it can't open connections yet. Make sure your
  node goes through the appropriate stages of the lifecycle.

* Invalid activation code — This happens during activation. Either the user made
  a mistake when entering the activation code, or the code has expired. Prompt
  the user to retry.

* Peer rejected / unavailable — You've tried to contact a peer that doesn't
  exist (rejected) or currently isn't connected to the hub (unavailable). In the
  latter case, you can either tell the user or somehow wake up the peer first
  (Wispers Files does this using Firebase Cloud Messaging).

* Hub unreachable — Either the network between the node and the Wispers hub is
  having problems, or the hub itself is down. Please follow the usual retrying
  best practices (i.e. don't retry-spam us, thank you!)

* Unauthenticated (node removed server-side) — This happens if the current node
  has been revoked and removed from its connectivity group, or the connectivity
  group itself has gone away. You should clear local state (logout) and restart
  the node lifecycle.


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
