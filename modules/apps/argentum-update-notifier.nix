{ config, lib, pkgs, ... }:

let
  cfg = config.apps.argentum-update-notifier;

  argentum-app-store = pkgs.callPackage ../../app-store { };

  # Runs every $interval. If `flatpak remote-ls --updates` lists ≥1 ref,
  # surface a libnotify notification with an "Open" action that launches
  # the App Store on the Updates page. `notify-send --wait --action=open=Open`
  # blocks until the user dismisses or clicks; on click the action id is
  # printed to stdout, which we use to decide whether to launch the store.
  notifier-script = pkgs.writeShellScript "argentum-update-notifier" ''
    set -eu
    updates=$(${pkgs.flatpak}/bin/flatpak remote-ls --updates --user --columns=ref 2>/dev/null | grep -c . || true)
    if [ "$updates" -gt 0 ]; then
      action=$(${pkgs.libnotify}/bin/notify-send \
        --app-name="App Store" \
        --icon=system-software-install \
        --action="open=Open" \
        --wait \
        "$updates app update(s) available" \
        "Click Open to review and update.")
      if [ "$action" = "open" ]; then
        ${argentum-app-store}/bin/argentum-app-store --page updates &
      fi
    fi
  '';
in {
  options.apps.argentum-update-notifier = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Per-user systemd timer that checks Flathub for app updates and surfaces a desktop notification.";
    };
    interval = lib.mkOption {
      type = lib.types.str;
      default = "4h";
      description = "OnUnitActiveSec interval. Common: 1h, 4h, 12h, daily.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.argentum-update-notifier = {
      description = "argentumOS — check Flathub for app updates and notify the user.";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${notifier-script}";
      };
    };

    systemd.user.timers.argentum-update-notifier = {
      description = "Run argentum-update-notifier on a regular cadence.";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnBootSec = "5min";
        OnUnitActiveSec = cfg.interval;
        Persistent = true;
      };
    };
  };
}
