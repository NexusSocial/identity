# Maps to the appropriate flake inputs based on the `system`.
{ inputs-raw, system }:
let
  isDarwin = (system == "aarch64-darwin" || system == "x86_64-darwin");
in
{
  inherit (inputs-raw) fenix nixos-generators home-manager deploy-rs;
  self = inputs-raw.self;
  nixpkgs = if isDarwin then inputs-raw.nixpkgs-24_11-darwin else inputs-raw.nixos-24_11;
  nixpkgs-unstable = if isDarwin then inputs-raw.nixpkgs-unstable else inputs-raw.nixos-unstable;
  # fenix = if isDarwin then inputs-raw.fenix-darwin else inputs-raw.fenix-linux;
}
