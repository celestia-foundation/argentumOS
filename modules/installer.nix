{ config, lib, pkgs, ... }:

let
  cfg = config.argentumOS;
  installer = pkgs.callPackage ../installer { };

  # Wrap `calamares` so it always loads our config dir. Upstream Calamares
  # hardcodes its search paths to its own Nix store prefix and ignores
  # /etc/calamares unless told via -c, so the unwrapped binary cannot find
  # settings.conf or branding/argentum/ that we install via environment.etc.
  # Qt 6.5+ requires libxcb-cursor at runtime to load the xcb platform plugin;
  # the calamares package in current nixpkgs does not pull it into its runtime
  # closure, so the wrapper injects it via LD_LIBRARY_PATH.
  calamaresRuntimeLibs = lib.makeLibraryPath [
    pkgs.xorg.xcbutilcursor
  ];

  # Wrap calamares so a plain `calamares` invocation:
  #   1. Always uses /etc/calamares as the config directory (-c).
  #   2. Has libxcb-cursor available for the Qt xcb platform plugin.
  #   3. Auto-elevates to root via `sudo -E` when run as a non-root user,
  #      preserving DISPLAY / XAUTHORITY so the elevated process can still
  #      connect to the live user's X server. This is required because the
  #      welcome module's `root` check and KPMcore's block-device probing
  #      both need euid=0.
  #
  # Elevation goes through /run/wrappers/bin/sudo, not ${pkgs.sudo}/bin/sudo:
  # Nix strips the setuid bit on store files, so the store-path binary cannot
  # change uid. The NixOS sudo module installs a setuid wrapper at
  # /run/wrappers/bin/sudo (mode 4755), and that is the only sudo on a NixOS
  # system that actually elevates. The autostart entry and the start-menu
  # launcher both reach calamares as a non-root user, so without this they
  # silently fail before the UI appears.
  calamaresLauncher = pkgs.writeShellScript "calamares-argentum" ''
    set -e
    export LD_LIBRARY_PATH="${calamaresRuntimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    if [ "$(id -u)" -ne 0 ]; then
      exec /run/wrappers/bin/sudo \
        --preserve-env=DISPLAY,XAUTHORITY,XDG_RUNTIME_DIR,WAYLAND_DISPLAY,LD_LIBRARY_PATH \
        ${pkgs.calamares}/bin/calamares -c /etc/calamares "$@"
    fi
    exec ${pkgs.calamares}/bin/calamares -c /etc/calamares "$@"
  '';

  # Upstream calamares.desktop has Exec=sh -c "pkexec calamares" which strips
  # DISPLAY and breaks. Replace with plain `calamares` so it routes through
  # our auto-elevating wrapper.
  calamaresDesktop = pkgs.writeText "calamares.desktop" ''
    [Desktop Entry]
    Type=Application
    Version=1.0
    Name=Install argentumOS
    GenericName=System Installer
    Keywords=calamares;system;installer;argentum;
    TryExec=calamares
    Exec=calamares
    Comment=Install argentumOS to your computer
    Icon=system-software-install
    Terminal=false
    StartupNotify=true
    Categories=Qt;System;
    X-AppStream-Ignore=true
  '';

  # symlinkJoin re-exposes the whole pkgs.calamares tree (share/applications,
  # share/icons, lib/calamares, …) so Cinnamon's start menu still finds the
  # entry and Calamares' own resources resolve. postBuild replaces
  # bin/calamares with our launcher and the upstream menu .desktop with a
  # corrected one.
  calamaresWrapped = pkgs.symlinkJoin {
    name = "calamares-argentum-${pkgs.calamares.version or "wrapped"}";
    paths = [ pkgs.calamares ];
    postBuild = ''
      rm -f $out/bin/calamares
      cp ${calamaresLauncher} $out/bin/calamares
      chmod +x $out/bin/calamares

      rm -f $out/share/applications/calamares.desktop
      cp ${calamaresDesktop} $out/share/applications/calamares.desktop
      chmod 644 $out/share/applications/calamares.desktop
    '';
  };

  # Combine the source modules/ tree with the installed.nix aggregator into
  # one store path, so a single environment.etc entry can expose them. Doing
  # this in two layered etc entries fails: the source path is read-only and
  # NixOS' /etc activation can't drop a new file inside it.
  argentumOSModules = pkgs.runCommand "argentumOS-modules" { } ''
    cp -r ${../modules} $out
    chmod -R u+w $out
    cat > $out/installed.nix <<'EOF'
    { config, lib, pkgs, ... }: {
      imports = [
        ./boot
        ./desktop
        ./apps/flatpak.nix
        ./wine
        ./branding
        ./installer.nix
      ];
      argentumOS.isLiveISO = false;
    }
    EOF
  '';
in
{
  options.argentumOS.isLiveISO = lib.mkOption {
    type = lib.types.bool;
    default = false;
    description = ''
      When true, ship the Calamares installer (with argentumOS branding and
      the custom nixos-install job) and autostart it on Cinnamon login.
      Set automatically by flake.nix only for the argentumISO configuration;
      always false on the installed system.
    '';
  };

  config = lib.mkIf cfg.isLiveISO {
    # `kdePackages.kpmcore` ships alongside calamares for a reason: its
    # `libexec/kpmcore_externalcommand` helper is what kpmcore execs (via
    # polkit) to do every block-device read, including the device enumeration
    # the partition module runs to decide whether any disk is installable.
    # pkgs.calamares only pulls kpmcore in as a buildInput, so the helper
    # binary and the `org.kde.kpmcore.externalcommand.policy` file don't end
    # up under /run/current-system/sw/{libexec,share/polkit-1/actions} — and
    # without them the partition module fails its "partitions" requirement
    # silently, which surfaces as "There are no partitions to install on."
    # Listing kpmcore here gets both artifacts onto the live ISO and into
    # polkit's policy search path.
    environment.systemPackages = [
      calamaresWrapped
      installer
      pkgs.kdePackages.kpmcore
    ];

    # Calamares' default search path includes /etc/calamares and
    # /run/current-system/sw/share/calamares — wiring the installer derivation's
    # outputs through environment.etc plus systemPackages covers both.
    environment.etc."calamares/settings.conf".source =
      "${installer}/etc/calamares/settings.conf";
    environment.etc."calamares/branding/argentum".source =
      "${installer}/share/calamares/branding/argentum";
    environment.etc."calamares/modules/nixos-install".source =
      "${installer}/share/calamares/modules/nixos-install";
    environment.etc."calamares/modules/welcome.conf".source =
      "${installer}/etc/calamares/modules/welcome.conf";
    environment.etc."calamares/modules/partition.conf".source =
      "${installer}/etc/calamares/modules/partition.conf";
    # -c /etc/calamares makes Calamares treat that as its application data
    # dir, so the qml/ subtree must live there too (not just under share/).
    environment.etc."calamares/qml".source =
      "${installer}/share/calamares/qml";
    environment.etc."xdg/autostart/argentum-installer.desktop".source =
      "${installer}/etc/xdg/autostart/argentum-installer.desktop";

    # Ship the argentumOS modules tree (with installed.nix aggregator baked in)
    # on the live ISO so the nixos-install job module can copy it into
    # /mnt/etc/nixos/argentumOS-modules/.
    environment.etc."argentumOS/modules".source = argentumOSModules;

    # Calamares relies on polkit for privilege elevation. Cinnamon brings an
    # agent in via the desktop module; enable polkit explicitly here so the
    # live ISO never depends on that ordering.
    security.polkit.enable = true;

    # The live `nixos` user has an empty password, which polkit rejects, so a
    # manual `calamares` run would prompt and fail. Grant passwordless YES to
    # wheel on the live ISO — this is the standard pattern for installer ISOs
    # and only ships when isLiveISO = true (never on the installed system).
    #
    # Root is matched explicitly in addition to wheel: after the launcher's
    # sudo elevation the calamares process runs as uid=0, and root is not a
    # member of wheel by default. KPMcore 24+ routes every block-device
    # operation through its polkit-gated `kpmcore_externalcommand` helper
    # even when the caller is already root, so without a root match the
    # action falls back to `auth_admin_keep`, the helper is denied, and the
    # welcome / partition screens report "no partitions to install on".
    security.polkit.extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (subject.isInGroup("wheel") || subject.user == "root") {
          return polkit.Result.YES;
        }
      });
    '';

    # `wheel` so the calamares wrapper can sudo -E to root without prompting.
    # `disk` would let an unprivileged process read block devices, but in
    # practice we elevate before doing any device work, so disk is not
    # strictly required — kept here as a defensive measure.
    users.users.nixos.extraGroups = [ "wheel" "disk" ];

    # Passwordless sudo for wheel — the live `nixos` user has an empty
    # password, so the default sudo prompt would fail. This pairs with the
    # calamares wrapper's `sudo -E` auto-elevation.
    security.sudo.wheelNeedsPassword = false;
  };
}
