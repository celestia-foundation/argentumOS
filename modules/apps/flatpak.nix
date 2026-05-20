{ config, lib, pkgs, ... }:

let
  cfg = config.apps.flatpak;
in {
  options.apps.flatpak = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable Flatpak — the primary app delivery mechanism for argentumOS.";
    };
  };

  config = lib.mkIf cfg.enable {
    services.flatpak.enable = true;

    xdg.portal = {
      enable = true;
      extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    };

    systemd.services.argentum-flathub-init = {
      description = "argentumOS — register the Flathub remote";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      path = [ pkgs.flatpak ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
      script = ''
        flatpak remote-add --if-not-exists flathub \
          https://dl.flathub.org/repo/flathub.flatpakrepo
      '';
    };
  };
}
