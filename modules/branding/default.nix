{ config, lib, pkgs, ... }:

let
  # Keep the wallpaper asset inside the modules tree so that the same
  # relative path resolves both during the live-ISO build *and* after the
  # Calamares nixos-install job copies modules/ into the target as
  # /mnt/etc/nixos/argentumOS-modules/. With the previous repo-root path
  # (`../../argentum.jpeg`) the copied tree pointed one directory above
  # itself — outside the copy — and `nixos-install` aborted with
  # `path '…/etc/nixos/argentum.jpeg' does not exist`.
  wallpaper = ./argentum.jpeg;
in {
  system.nixos.distroName = "argentumOS";
  system.nixos.distroId = "argentumos";

  system.nixos.label = lib.mkForce "argentumOS";

  environment.etc."os-release".text = lib.mkForce ''
    NAME="argentumOS"
    ID=argentumos
    ID_LIKE=nixos
    PRETTY_NAME="argentumOS"
    HOME_URL="https://github.com/"
    LOGO=argentum
    ANSI_COLOR="1;34"
  '';

  environment.etc."argentumos/wallpaper.jpeg".source = wallpaper;

  environment.etc."backgrounds/argentum.jpeg".source = wallpaper;

  environment.etc."argentumos/branding/wallpaper.jpeg".source = wallpaper;

  environment.etc."cinnamon-background-properties/argentumos.xml".text = ''
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE wallpapers SYSTEM "gnome-wp-list.dtd">
    <wallpapers>
      <wallpaper deleted="false">
        <name>argentumOS</name>
        <filename>/etc/backgrounds/argentum.jpeg</filename>
        <options>zoom</options>
        <pcolor>#0f1218</pcolor>
        <scolor>#0f1218</scolor>
        <shade_type>solid</shade_type>
      </wallpaper>
    </wallpapers>
  '';
}
