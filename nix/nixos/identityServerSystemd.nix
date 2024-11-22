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

  systemd.services.identity-server = {
    description = "NexusSocial Identity Server";
    after = [ "podman.service" ];
    requires = [ "podman.service" ];
    serviceConfig = {
      TimeoutStartSec = 0;
      Restart = "always";
      ExecStartPre = [
        "-/usr/bin/env podman stop %n"
        "-/usr/bin/env podman rm %n"
        "-/usr/bin/env podman pull ghcr.io/nexussocial/identity-server:latest"
      ];
      ExecStart = "/usr/bin/env podman run --rm --name %n identity-server";
    };
    wantedBy = [ "multi-user.target" ];
  };

}
