{ config, lib, pkgs, ... }:

let
  cfg = config.programs.argentum-settings;

  argentum-settings = pkgs.callPackage ../settings-panel { };

  # The Cinnamon panel and several built-in applets invoke `cinnamon-settings`
  # directly with subcommand arguments (e.g. `cinnamon-settings display`).
  # A `.desktop` override alone misses those. This shim forwards every
  # invocation to argentum-settings with the matching `--page` flag, so every
  # entry point into "settings" lands in the new panel.
  cinnamon-settings-shim = pkgs.writeShellScriptBin "cinnamon-settings" ''
    page="''${1:-appearance}"
    case "$page" in
      sound|sound-input|sound-output) page=sound ;;
      backgrounds|themes|fonts)        page=appearance ;;
      display|monitor)                 page=display ;;
      network|wifi|vpn)                page=network ;;
      user|users|accounts)             page=users ;;
      flatpak|applications|software)   page=software ;;
      datetime|time|calendar)          page=date-time ;;
      hostname|info|details|system)    page=system ;;
    esac
    exec ${argentum-settings}/bin/argentum-settings --page "$page"
  '';
in {
  options.programs.argentum-settings = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable the native argentumOS system settings panel and shadow Cinnamon's.";
    };
  };

  config = lib.mkIf cfg.enable {
    # Order matters: cinnamon-settings-shim must come before the cinnamon
    # package on PATH so its `cinnamon-settings` wins. `environment.systemPackages`
    # resolves earlier list entries first when building the system profile.
    #
    # `zenity` backs `widgets::prompt` (hostname, password, add-remote prompts)
    # — must be present on PATH at runtime. `pulseaudio` provides `pactl` for
    # the Sound page (PipeWire's pulse compat layer accepts pactl commands).
    # `networkmanager` provides `nmcli` for the WiFi connect flow.
    environment.systemPackages = [
      cinnamon-settings-shim
      argentum-settings
      pkgs.zenity
      pkgs.pulseaudio   # ships pactl
      pkgs.networkmanager  # ships nmcli
    ];

    # Replace the .desktop entry so menu launchers / xdg-open also route to
    # argentum-settings. `xdg.desktopEntries` is the user-facing analogue of
    # `environment.etc."xdg/applications/cinnamon-settings.desktop"` and is the
    # blessed knob.
    xdg.mime.defaultApplications = {
      "application/x-cinnamon-settings" = "argentum-settings.desktop";
    };

    environment.etc."xdg/applications/cinnamon-settings.desktop".text = ''
      [Desktop Entry]
      Name=System Settings
      Comment=argentumOS system settings
      Exec=argentum-settings
      Icon=preferences-system
      Terminal=false
      Type=Application
      Categories=System;Settings;
      StartupNotify=false
      X-GNOME-Settings-Panel=argentum-settings
    '';

    # Polkit rules so the System page's `pkexec hostnamectl set-hostname` and
    # `pkexec nixos-rebuild switch --upgrade` prompts work for the wheel group
    # without round-tripping a passwordless rule into general sudo.
    security.polkit.extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (subject.isInGroup("wheel") && (
              action.id == "org.freedesktop.hostname1.set-hostname" ||
              action.id.indexOf("org.freedesktop.policykit.exec") == 0
            )) {
          return polkit.Result.AUTH_ADMIN_KEEP;
        }
      });
    '';
  };
}
