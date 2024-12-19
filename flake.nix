{
  description = "NexusSocial/identity repo";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    # Linux
    nixos-24_11.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixos-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators/7c60ba4bc8d6aa2ba3e5b0f6ceb9fc07bc261565";
      inputs.nixpkgs.follows = "nixos-24_11";
    };
    home-manager = {
      url = "github:nix-community/home-manager/release-24.11";
      inputs.nixpkgs.follows = "nixos-24_11";
    };

    #Darwin
    nixpkgs-24_11-darwin.url = "github:NixOS/nixpkgs/nixpkgs-24.11-darwin";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };

    # For accessing `deploy-rs`'s utility Nix functions
    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
  };

  outputs = inputs-raw@{ flake-utils, ... }:
    let
      # All systems we may care about evaluating nixpkgs for
      systems = with flake-utils.lib.system; [ x86_64-linux aarch64-linux aarch64-darwin x86_64-darwin ];
      perSystem = (system: rec {
        inputs = import ./nix/inputs.nix { inherit inputs-raw system; };
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            ((import nix/overlays/nixpkgs-unstable.nix) { inherit inputs; })
          ];
          config = {
            # allowUnfree = true;
          };
        };
      });
      # This `s` helper variable caches each system we care about in one spot
      inherit (flake-utils.lib.eachSystem systems (system: { s = perSystem system; })) s;
    in
    # System-specific stuff goes in here, by using the flake-utils helper functions
    flake-utils.lib.eachSystem systems
      (system:
        let
          inherit (s.${system}) pkgs inputs;
        in
        {
          devShells = import ./nix/devShells.nix { inherit system pkgs inputs; };
          packages = {
            deploy-rs = inputs.deploy-rs.packages.${system}.deploy-rs;
          };
          formatter = pkgs.nixpkgs-fmt;
        }
      )
    # Next, concatenate deploy-rs stuff to the flake
    // import ./nix/deploy-rs.nix {
      inputs = s."x86_64-linux".inputs;
    }
    # Concatenate NixOS stuff
    // {
      nixosConfigurations = import ./nix/nixos/nixosConfigurations.nix { inherit s; };
    };
}
