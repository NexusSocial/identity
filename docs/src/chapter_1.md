# Technical Explanation of `did:yeet` (Placeholder name)

> [!NOTE]
> For a high level overview and user stories, see a different document (TODO).
> Furthermore, this is WIP, and should neither be treated as a reference nor
> a canonical source of truth for the latest design.

## Identifier syntax
`did:yeet` is formatted:

`did:yeet:deadbeefdeadbeef`

in the example above, the last segment after the `:` is WIP. *Probably*, we will
use a [Multibase][multibase] and [Multicodec][multicodec] encoded, base58-btc sha256 hash.

> [!NOTE] Why did we choose base58-btc?
>
> Its because its what [did:key][did-key] uses and because it doesn't have any
> non-alphanumeric characters (unlike base64). It also omits commonly mistaken
> characters like `0Iol`. There are also excellent high performance encoding
> and decoding libraries, with libraries in both [rust][bs58] and c#.

```
did-yeet-format := did:yeet:<mb-value>
mb-value       := z[a-km-zA-HJ-NP-Z1-9]+
```
You can think of this like this series of transformations:
```
did-yeet-format := did:yeet:MULTIBASE(base58-btc, MULTICODEC(hash-function-type, cbor-user-genesis-bytes))
```
Where `cbor-user-genesis-bytes` are the bytes of the [DAG-CBOR][dag-cbor]
encoding of the user's genesis entry for their account. For more info on what
`cbor-user-genesis-bytes` is, see the section on "user ledger".

TODO: Is there a multicodec for DAG-CBOR we should include too, or is that
overkill.

## User Ledger

The user ledger is similar in concept to the ledger in [`did:plc`][did-plc].
Each user has their own, separate ledger. There are a number of entry types
in the ledger.

### Genesis Entry

The genesis entry represents the account state at the time of creation. Every
user ledger MUST have exactly one genesis entry, which is immutable because
changing it would change the DID itself.

The genesis entry can contain multiple keys, all of which hold equal and
absolute power over the account. It is not possible to revoke or change any of
these DIDs.

The following are the genesis entry contents:
* `version`: A version integer for the Genesis Entry. For now, only `0` is valid.
* `signed-by`: A key id that signed this entry (i.e. it is self-referential).
* `signature`: The signature of this entry. The payload signed should match the
  sha256 hash of the DAG-CBOR encoded form of the current entry, but with the
  `signature` field zeroed out.
* `keys`: A structure that looks like a JSON Web Key Set. The `kid` (key ID)
  parameter of each of the keys is required, and MUST be unique amongst all keys
  in the genesis entry. For now, only ed25519 keys are supported.

### Delegation Entry

This entry is optional. It is a way for the user to delegate responsiblities of
the account to subkeys, or to update the account to change to new keys.

TODO: See if we should use DIDs instead of JWKs

The delegation entry is a set, where each member of the set contains the
following fields:
* `key`: A structure that looks like a JSON Web Key. The `kid` (key ID)
  parameter of each of the keys is required, and MUST be unique amongst all keys
  in both the genesis and delegation entries. For now, only ed25519 keys are
  supported.
* `parent`: The `kid` (key ID) of the key that enrolled this one.
* `revoked-by` (Optional): The `kid` of the key that revoked this key, if it
  was revoked.
* `capabilities`: A list of capabilities of this key. TODO: Figure out how to
  do capabilities like signing, sibling revocation, etc.
* `signature`: The signature from the parent key, enrolling this child key. The
  payload signed should match the sha256 hash of the DAG-CBOR encoded form of the
  child key map object, but with the `signature` field zeroed out.

### Document Entry

This entry contains data that is equivalent to the remaining properties of a DID document.
It will be merged with the DID document produced by resolving the other entries in the
keychain.

TODO: Demonstrate how this works


## User Ledger Verification

In order to trust that a ledger is valid for a given DID, it is necessary to verify that ledger.

The algorithm to verify a ledger is:
1. The user ledger (*which is not the same thing as the DID Document*) for a
   given `did:yeet` needs to be retrieved one or more peers. If retrieving from
   multiple peers, it is necessary that all genesis entries are identical.
1. Next, all delegation entries must be merged into a single delegation entry.
   When merging the value of `revoked-by`, it is acceptable for there to be 0 or more "null" values and at most one "non-null" value - the non-null value should be chosen. If there are more than one non-null value for a given key's `revoked-by`, the user ledger is invalid.
1. The genesis entry in the ledger is encoded into its canonical DAG-CBOR representation, yielding
   `cbor-user-genesis-bytes`.
1. `cbor-user-genesis-bytes` are fed into the series of hashing and encoding
   functions specified in the "identifier syntax" section.
1. The resulting encoded value should match the original did for the user. If
   there is any mismatch, this ledger does not correspond to the expected DID and
   verification should fail.
1. Next, a directed graph of the keys are formed using the `parent` field of
   the keys in the delegation entry. If the graph has any cycles, or if any of
   the "heads" of the graph are not keys from the genesis entry, the user ledger
   is rejected as invalid.
1. Next, a "virtual node" is added as a parent of all the heads of the graph
   such that there is now only one head node. The distance of each node to this
   singular head node is measured. Any nodes that have been "revoked", and whose
   `revoked-by` is at the same distance from the head as them, are siblings with
   the node that revoked them. Therefore if the node indicated by `revoked-by`
   does not have the "revoke sibling" capability, the user ledger is rejected as
   invalid.
1. Finally, every signature in the user ledger must be verified.

### Limitations of ledger verification

As long as users retain custody of their keys or give them only to trusted
parties, the hash and signatures stored in the ledger would be enough to
demonstrate its validity. This is because all keys were signed by parent keys, eventually
going all the way back to the first keys in the genesis entry.

However because the entries of the ledger can be updated over time, there is no
guarantee that any partiuclar ledger is actually up to date with the
"canonical" version. Because of this, it might contain keys that that the
end-user has since revoked.

Luckily, the ledger is guaranteed to eventually propagate throughout the network and
get the latest updates even in the face of "byzantine" or adversarial/incompatible
peers, as long as any one peer on the network can talk to at least another "correct" (non-byzantine)
peer at some point before their nodes start rejecting updates due to age.

Unfortuantely it is not possible to know if a node is trustworthy/compliant or not,
until finally connecting to a "correct" node.

For this reason, clients that want to be more certain that they are able to
observe key revocations, should connect to several nodes run by parties
unlikely to collude, or have a known server they trust that can do this on
their behalf. Luckily, these nodes cannot *lie*, they can only *omit*
information. And ultimately, users' accounts are still controlled by private keys, so
performing an attack requires *both* access to the private keys, and if the keys were
revoked, collaboration with all nodes that the verifying party connects to.

## User Ledger Propagation

TODO: Explain how user ledgers propagate throughout the yeet network, via
BFT-CRDTs, update validation functions, and .

## Security Considerations

### Differences in key revocation

If keys are leaked, it is possible to regain control of the account if the
legitimate user has a parent key of the leaked key. In this case, the parent
key has unambiguous ability to sign a key revocation, setting the `revoked-by`
field of the compromised child key.

If however a parent key is unavailable (such as if the keys in the genesis
entry have been lost), it is also possible to revoke a sibling key *if* the legitimate user
has access to a key at the same depth and with the "revoke siblings" capability.
Using this capability, the user can revoke the compromised or lost key, rendering
it useless.

However, for either revocation to be useful, clients need to actually observe a user ledger
with those revocations. A client might not observe this iff they never connect to a
non-byzantine node.

[multibase]: https://github.com/multiformats/multibase
[multicodec]: https://github.com/multiformats/multicodec/tree/master
[did-key]: https://w3c-ccg.github.io/did-method-key/
[did-plc]: https://github.com/did-method-plc/did-method-plc
[dag-cbor]: https://ipld.io/docs/codecs/known/dag-cbor/
[dag]: https://en.wikipedia.org/wiki/Directed_acyclic_graph
[bs58]: https://docs.rs/bs58/latest/bs58/
