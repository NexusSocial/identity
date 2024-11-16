# Defines all nixosConfigurations for the flake
{ s }:
let
  inherit (s."x86_64-linux") pkgs inputs;

  nixosConfig =
    { hostname
    , username
    , nixosConfigFile
    , homeConfigFile
    }:
    inputs.nixpkgs.lib.nixosSystem
      rec {
        inherit system;
        specialArgs = {
          inherit hostname username pkgs; modulesPath = "${inputs.nixpkgs}/nixos/modules";
        };
        modules = [
          nixosConfigFile
          # setup home-manager
          inputs.home-manager.nixosModules.home-manager
          {
            home-manager = {
              useGlobalPkgs = true;
              useUserPackages = true;
              # include the home-manager module
              users.${username} = import homeConfigFile;
              extraSpecialArgs = rec {
                inherit username pkgs;
              };
            };
            # https://github.com/nix-community/home-manager/issues/4026
            # users.users.${username}.home = s.${system}.pkgs.lib.mkForce "/Users/${username}";
          }
        ];
      };
in
{
  "servers-us-east-1" = nixosConfig
    {
      hostname = "servers-us-east-1";
      username = "admin";
      nixosConfigFile = ./servers-us-east-1/configuration.nix;
      homeConfigFile = ./home.nix;
    };
}
