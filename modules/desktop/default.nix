{ config, lib, pkgs, ... }:

{
  imports = [
    ./theme
  ];

  services.xserver.enable = true;

  services.xserver.displayManager.lightdm = {
    enable = true;
    greeters.slick.enable = true;
  };

  services.xserver.desktopManager.cinnamon.enable = true;

  services.cinnamon.apps.enable = true;

  services.xserver.xkb.layout = "us";

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };

  services.printing.enable = true;
}
