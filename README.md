# Wispers Connect

Wispers Connect is an application-level VPN library. It connects software
running on different devices over the Internet with NAT-traversing,
peer-to-peer connections. A central Hub handles coordination and signaling,
but data flows directly between nodes. End-to-end encryption ensures that
not even the Hub can read your traffic.

You can embed Wispers Connect as a library (Rust, Kotlin/Android, Go) or
run it as a sidecar alongside your existing processes.

## Key concepts

<!-- TODO: short (2-3 sentence) explanation of each concept, linking to
     HOW_IT_WORKS.md for the full picture. Cover:
     - Nodes
     - Connectivity Groups
     - The Hub (signaling only, never sees data)
     - Activation & the Roster (transitive trust via pairing codes)
     - Domains & Organisations (integrator concepts)
     See connect/DESIGN.md "Concepts & names" for source material. -->

## Quick start

<!-- TODO: minimal end-to-end example showing the three steps:
     1. Register a node (get a token from the integrator, call register())
     2. Activate (pair two nodes using a code)
     3. Connect (open a QUIC stream to a peer)
     Show code for one wrapper (Rust or Go) and link to HOW_TO_USE.md for
     all wrappers and more examples. -->

## Documentation

- **[How it works](docs/HOW_IT_WORKS.md)** — Transport, security model,
  and protocol design
- **[How to use it](docs/HOW_TO_USE.md)** — Integration guide with
  examples for each wrapper
- **[Internals](docs/INTERNALS.md)** — Code map, module responsibilities,
  and key types

## Building

<!-- TODO: cover building the Rust library, cross-compiling for Android
     (cargo-ndk), and linking from Go/Kotlin. Keep it short — point to
     HOW_TO_USE.md for wrapper-specific setup. -->

## License

<!-- TODO -->
