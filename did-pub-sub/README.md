# `did-pub-sub` - Publish/Subscribe messaging with DIDs

Peers address each other using Decentralized Identifiers (DIDs). They can publish to
topics, and subscribe to those topics. The entire network is peer to peer with no
central node. You simply need a list of the peers you care about.

This peer list may come from, for example, a friends list, or the list of players
connected to your server, or something else - its up to you.

Technology used:
* [did-pkarr](../did-pkarr/) - A Decentralized Identifier built on top of Bittorrent.
* [iroh][iroh] - Peer-to-peer QUIC
- [iroh-gossip][iroh-gossip] - gossip protocol on top of iroh.

[iroh]: https://iroh.computer
[iroh-gossip]: https://docs.rs/iroh-gossip
