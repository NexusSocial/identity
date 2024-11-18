# Defines all devShells for the flake
{ system, pkgs, inputs }:
let
  inherit (inputs) fenix;
  rustToolchain = fenix.packages.${system}.fromToolchainFile {
    file = ../rust-toolchain.toml;
    sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
  };
  rustPlatform = pkgs.makeRustPlatform {
    inherit (rustToolchain) cargo rustc;
  };
in
{
  default = pkgs.mkShell {
    # These programs be available to the dev shell
    buildInputs = (with pkgs; [
      nixpkgs-fmt
    ]) ++ [
      rustToolchain
      rustPlatform.bindgenHook
      # fenix.packages.${system}.rust-analyzer
    ];

    # Hook the shell to set custom env vars
    shellHook = ''
      # FOOBAR=1
    '';
  };
}
