# This is your system's configuration file.
# Use this to configure your system environment (it replaces /etc/nixos/configuration.nix)
{ inputs
, hostname
, username
, lib
, config
, pkgs
, ...
}: {
  # You can import other NixOS modules here
  imports = [ ];

  # BEGIN Recommendations from the linode article
  boot.loader.grub.forceInstall = true;
  boot.loader.grub.device = "nodev";
  boot.loader.timeout = 10;

  # Enables linode's LISH support
  boot.kernelParams = [ "console=ttyS0,19200n8" ];
  boot.loader.grub.extraConfig = ''
    serial --speed=19200 --unit=0 --word=8 --parity=no --stop=1;
    terminal_input serial;
    terminal_output serial
  '';

  networking = {
    usePredictableInterfaceNames = false;
    useDHCP = lib.mkForce false; # Disable DHCP globally as we will not need it.
    # required for ssh?
    interfaces.eth0.useDHCP = true;
  };

  fileSystems."/" =
    {
      device = "/dev/sda";
      fsType = "ext4";
    };

  swapDevices =
    [{ device = "/dev/sdb"; }];
  # END 

  nixpkgs.flake = {
    setFlakeRegistry = true;
    setNixPath = true;
  };

  nix =
    let
      flakeInputs = lib.filterAttrs (_: lib.isType "flake") inputs;
    in
    {
      package = pkgs.nix;
      settings = {
        # Enable flakes and new 'nix' command
        experimental-features = "nix-command flakes";
        trusted-users = [
          "root"
          "@admin"
          username
        ];
      };
      nixPath = lib.mkForce [ "nixpkgs=flake:nixpkgs" ];
      # Opinionated: disable channels
      channel.enable = false;
    };

  networking.hostName = hostname;

  users.groups = {
    plugdev = { };
  };
  users.users = {
    ${username} = {
      isNormalUser = true;
      openssh.authorizedKeys.keys = [
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBLmHbuCMFpOKYvzMOpTOF+iMX9rrY6Y0naarcbWUV8G ryan@ryan-laptop"
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIL6iX+gVgpmt5qj+VPTtk/SlAjlZTOXH2Ysdao0MLfNS ci@NexusSocial/identity"
      ];
      # TODO: Be sure to add any other groups you need (such as networkmanager, audio, docker, etc)
      extraGroups = [
        "wheel"
      ];
    };
  };
  users.mutableUsers = false;
  security.sudo.wheelNeedsPassword = false; # TODO: Change to true once satisfied with config

  # This setups a SSH server. Very important if you're setting up a headless system.
  # Feel free to remove if you don't need it.
  services.openssh = {
    enable = true;
    settings = {
      # Opinionated: forbid root login through SSH.
      # PermitRootLogin = "no"; # didnt work with nixos-generators
      # Opinionated: use keys only.
      # Remove if you want to SSH using passwords
      PasswordAuthentication = false;
    };
  };


  environment.systemPackages = with pkgs; [
    neovim
    ripgrep

    # Recommended by https://www.linode.com/docs/guides/install-nixos-on-linode/#install-diagnostic-tools
    inetutils
    mtr
    sysstat

  ];

  # https://nixos.wiki/wiki/FAQ/When_do_I_update_stateVersion
  system.stateVersion = "24.05";
}
