{
  description = "NexusSocial/identity repo";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    # Linux
    nixos-24_05.url = "github:NixOS/nixpkgs/nixos-24.05";
    nixos-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixos-24_05";
    };

    #Darwin
    nixpkgs-24_05-darwin.url = "github:NixOS/nixpkgs/nixpkgs-24.05-darwin";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixos-24_05";
    };
  };

  outputs = raw-inputs@{ flake-utils, ... }:
    let
      # All systems we may care about evaluating nixpkgs for
      systems = with flake-utils.lib.system; [ x86_64-linux aarch64-linux aarch64-darwin x86_64-darwin ];
      perSystem = (system: rec {
        inputs = ./nix/inputs.nix { inherit raw-inputs system; };
        pkgs = import raw-inputs.nixpkgs-unstable {
          inherit system;
          overlays = [
            ((import nix/overlays/nixpkgs-unstable.nix) { inherit inputs; })
          ];
          config = {
            # allowUnfree = true;
          };
        };
      });
      # This helper variable caches each system we care about in one spot
      inherit (flake-utils.lib.eachSystem systems (system: { s = perSystem system; })) s;
    in
    # Now we can proceed with the "typical" way of doing flakes via flake-utils:
    flake-utils.lib.eachSystem systems
      (system:
        let
          inherit (s.${system}) pkgs inputs;
        in
        {
          formatter = pkgs.nixpkgs-fmt;
        }
      );
}
