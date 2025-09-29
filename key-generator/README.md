# No-std + alloc compatible recovery phrase generator for ed25519 private keys.

NOTE: no-std is not actually true rn due to `slip10_ed25519`, we will fork and fix that.

* Recovery phrases are randomly generated from 256 bits of entropy, producing a 24 word
  recovery phrase. This phrase supports several languages (English, Chinese, Japanese,
  etc), via the [BIP-39] alphabet.
* Recovery phrases are then combined with an *optional* password, to derive a private
  seed. This 512 bit seed is the final output of [BIP-39], and is passed to subsequent
  steps. Note that the password is never saved alongside the recovery phrase, that
  would defeat the point of the password. Users that are prone to forgetting their
  password should be prompted to omit the password. Note that the choice of a password
  cannot be changed later, without resulting in a new private key.
* ed25519 private keys are then derived using [SLIP-0010][SLIP-0010] from the private
  seed. This adds a bit of safety by ensuring that private keys in different curves
  using the same seed will not undermine each other (its unsafe to use the same values
  on different curves).

## Optional Features

* `export-pdf` feature enables generation of a PDF, to make it easy for users to print
  out an account "Recovery Kit"

[BIP-39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
[SLIP-0010]: https://github.com/satoshilabs/slips/blob/master/slip-0010.md
