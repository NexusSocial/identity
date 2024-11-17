{ inputs }: {
  deploy = {
    nodes.servers-us-east-1 = {
      hostname = "servers-us-east-1.socialvr.net";
      profiles.system = {
        user = "root"; # The user to deploy to, not necessarily the same as the SSH user.
        path = inputs.deploy-rs.lib.x86_64-linux.activate.nixos inputs.self.nixosConfigurations.servers-us-east-1;
      };
    };
    sshUser = "admin";

    # Timeout for profile activation.
    activationTimeout = 240;
    # Timeout for profile activation confirmation.
    confirmTimeout = 30;
  };

  # This is highly advised by deploy-rs, and will prevent many possible mistakes
  checks = builtins.mapAttrs (system: deployLib: deployLib.deployChecks inputs.self.deploy) inputs.deploy-rs.lib;
}
