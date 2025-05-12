# Defines all devShells for the flake
{ system, pkgs, inputs }:
let
  inherit (inputs) fenix;
  rustToolchain = fenix.packages.${system}.fromToolchainFile {
    file = ../rust-toolchain.toml;
    sha256 = "sha256-X/4ZBHO3iW0fOenQ3foEvscgAPJYl2abspaBThDOukI=";
  };
  rustPlatform = pkgs.makeRustPlatform {
    inherit (rustToolchain) cargo rustc;
  };
  dotnet = (with pkgs.dotnetCorePackages;
    # We will combine the two latest dotnet SDKs to give all tools time to
    # upgrade
    combinePackages [
      sdk_9_0
    ]);
in
{
  default = pkgs.mkShell {
    # These programs be available to the dev shell
    buildInputs = (with pkgs; [
      cargo-deny
      dotnet
      mdbook
      mdbook-mermaid
      nixpkgs-fmt
      roslyn-ls
    ]) ++ pkgs.lib.optional pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ] ++ [
      rustToolchain
      rustPlatform.bindgenHook
      # fenix.packages.${system}.rust-analyzer
    ];

    # Hook the shell to set custom env vars
    shellHook = ''
    '';
  };
}
