# did:pkarr - PKARR based Decentralized Identifiers

`did:pkarr` are [DID][did]s that are based on
[Public Key Addressable Resource Records](pkarr).

## Why use did:pkarr over other alternatives?

* Unlike `did:key`, it is mutable and has a richer set of data it can contain.
* Unlike `did:web`, it is fully decentralized.

## Example of a did:pkarr

`did:pkarr:47pjoycnsrfmxikm95jh13y88e8qnhzu5kungjpxyepgt7a8krpy`

The basic format is:
* `did:` - all Decentralized Identifiers start with this
* `pkarr:` - Indicates which type of DID this is - in our case, `did:pkarr`.
* `47pjoycnsrfmxikm95jh13y88e8qnhzu5kungjpxyepgt7a8krpy` - the public key.

## How does it work?

* Parse the pubkey out of the `did:pkarr:<public-key>`.
* Use [pkarr][pkarr] to look up the associated (signed) DNS TXT record.
* Convert that TXT record into a DID Document.

## Project Status

This project is experimental and prone to breaking changes. Be warned!

[did]: https://www.w3.org/TR/did-1.1/
[pkarr]: https://github.com/pubky/pkarr
