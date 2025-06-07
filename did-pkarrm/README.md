# did:pkarrm - PKARR(multiformats) based Decentralized Identifiers

`did:pkarrm` are [DID][did]s that are based on [Public Key Addressable Resource Records](dkarr).
The `m` stands for [Multiformats][multiformats] - unlike regular PKARR, the identifiers
use multiformats to self-describe the encoding and pubkey type.

`did:pkarrm` has a 1:1 mapping to PKARR addresses, meaning that all existing PKARR crates
can continue to be used

## Example of PKARRm

Example of a standard PKARR address: `
self-describing.
* `h` is the [multibase][multibase] encoding of `base32-z`
* `72` is the base32-z encoding of `0xED`, the [multicodec][multicodec] identifier for
  ED25519 pubkeys


## Project Status

This project is experimental and prone to breaking changes. It might not even be
functional. Be warned!

[pkarr]: https://github.com/pubky/pkarr
[did]: https://www.w3.org/TR/did-1.0/NexusSocial
