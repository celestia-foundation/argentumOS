{ config, lib, pkgs, ... }:

let
  gtkThemeName = "adw-gtk3-dark";
  iconThemeName = "Papirus-Dark";
  cinnamonThemeName = "Mint-Y-Dark-Aqua";
in {
  environment.systemPackages = with pkgs; [
    adw-gtk3
    papirus-icon-theme
    mint-themes
    mint-y-icons
  ];

  programs.dconf.enable = true;

  programs.dconf.profiles.user.databases = [{
    settings = {
      "org/cinnamon/desktop/interface" = {
        gtk-theme = gtkThemeName;
        icon-theme = iconThemeName;
        cursor-theme = "Adwaita";
      };

      "org/cinnamon/theme" = {
        name = cinnamonThemeName;
      };

      "org/cinnamon" = {
        panels-enabled = [ "1:0:bottom" ];
        panel-launchers = [
          "nemo.desktop"
          "firefox.desktop"
          "org.gnome.Terminal.desktop"
        ];
      };

      "org/cinnamon/desktop/screensaver" = {
        lock-enabled = false;
        idle-activation-enabled = false;
      };

      "org/cinnamon/desktop/session" = {
        idle-delay = lib.gvariant.mkUint32 0;
      };

      "org/cinnamon/desktop/background" = {
        picture-uri = "file:///etc/backgrounds/argentum.jpeg";
        picture-options = "zoom";
        primary-color = "#0f1218";
        secondary-color = "#0f1218";
        color-shading-type = "solid";
      };

      "org/cinnamon/desktop/background/slideshow" = {
        slideshow-enabled = false;
      };
    };
  }];
}
