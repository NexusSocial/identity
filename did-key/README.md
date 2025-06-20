# did:key support

[did:key][did-key] is a [Decentralized Identifier][did-core] that embeds the public key
directly in the DID.

## Why use the did:key method vs other DIDs?

Its *just a public key*.

## Why use this particular crate

Unlike other crates in the ecosystem, this one:

* Is *exclusively* parsing/deserialization (sans-io). You give us strings, we give you
  pubkey bytes.
* Generic across all public key types.
* Can be used with any cryptography library (we recommend [ed25519-dalek]).
* Lightweight - no unecessary dependencies or overcomplicated APIs.
* Supports no_std + alloc.

## Breaking Changes

This crate is v0.0.X, and may introduce breaking changes at any time, with any
frequency.

[did-core]: https://www.w3.org/TR/did-1.1/
[did-key]: https://w3c-ccg.github.io/did-key-spec/
[ed25519-dalek]: https://docs.rs/ed25519-dalek/
