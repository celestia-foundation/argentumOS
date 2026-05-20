{ lib
, runCommand
, writeText
  # Branding asset source-of-truth: same directory consumed by the Plymouth
  # theme derivation. Both derivations independently produce store paths
  # containing splash.png; the working-tree source is single.
, plymouthAssets ? ../modules/boot/argentum-plymouth/assets
}:

let
  # No OnlyShowIn= — XDG_CURRENT_DESKTOP can be reported as either "Cinnamon"
  # or "X-Cinnamon" depending on session version, and filtering on it drops
  # the entry silently. The live ISO only ships one desktop, so an unfiltered
  # autostart is correct.
  autostart = writeText "argentum-installer.desktop" ''
    [Desktop Entry]
    Type=Application
    Name=Install argentumOS
    Comment=Install argentumOS to your computer
    Exec=calamares
    Icon=system-software-install
    Terminal=false
    X-GNOME-Autostart-enabled=true
  '';
in
runCommand "argentum-installer" {
  meta = {
    description = "argentumOS Calamares installer configuration, branding, QML, and nixos-install job";
    platforms = lib.platforms.linux;
  };
} ''
  # /etc/calamares/settings.conf — discovered by Calamares' default search path.
  install -Dm644 ${./calamares/settings.conf} \
    $out/etc/calamares/settings.conf

  # /etc/xdg/autostart entry — modules/installer.nix symlinks it into /etc/xdg/autostart.
  install -Dm644 ${autostart} \
    $out/etc/xdg/autostart/argentum-installer.desktop

  # Branding directory.
  install -Dm644 ${./calamares/branding/argentum/branding.desc} \
    $out/share/calamares/branding/argentum/branding.desc
  install -Dm644 ${./calamares/branding/argentum/stylesheet.qss} \
    $out/share/calamares/branding/argentum/stylesheet.qss
  install -Dm644 ${./calamares/branding/argentum/show.qml} \
    $out/share/calamares/branding/argentum/show.qml
  install -Dm644 ${plymouthAssets}/splash.png \
    $out/share/calamares/branding/argentum/splash.png

  # Custom Calamares Python job module.
  install -Dm644 ${./calamares/modules/nixos-install/module.desc} \
    $out/share/calamares/modules/nixos-install/module.desc
  install -Dm755 ${./calamares/modules/nixos-install/main.py} \
    $out/share/calamares/modules/nixos-install/main.py

  # Per-module config files, picked up by Calamares from <config>/modules/
  # when -c /etc/calamares is in effect.
  install -Dm644 ${./calamares/modules/welcome.conf} \
    $out/etc/calamares/modules/welcome.conf
  install -Dm644 ${./calamares/modules/partition.conf} \
    $out/etc/calamares/modules/partition.conf

  # QML pages + components — available to Calamares as view-step modules via
  # the share/calamares/qml directory.
  mkdir -p $out/share/calamares/qml
  cp -r ${./qml}/. $out/share/calamares/qml/
''
