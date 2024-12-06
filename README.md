# NexusSocial Identity Repo

This serves as a monorepo for projects related to accounts and identity.

In particular, it hosts:
* [did-simple](/did-simple) - A simple pure rust and sans-io crate with minimal
  dependencies to work with did:key and did:web urls, signing, and verifying.
  did:web
* [identity-server](/identity-server) - A did:web HTTP server that implements a "sign
  in with google/meta" approach.

## First Time Setup

- Install [rustup](https://rustup.rs)
- Install [git lfs](https://git-lfs.com/) and run `git lfs install` and `git lfs pull`

## License

Unless otherwise specified, all code in this repository is dual-licensed under
either:

- MIT-0 License ([LICENSE-MIT-0](LICENSE-MIT-0))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you can select the license you prefer!

Any contribution intentionally submitted for inclusion in the work by you, shall be
dual licensed as above, without any additional terms or conditions.
