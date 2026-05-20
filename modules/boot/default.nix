{ config, lib, pkgs, ... }:

let
  argentumPlymouth = pkgs.callPackage ./argentum-plymouth { };
in {
  boot.loader.systemd-boot.enable = lib.mkDefault false;

  boot.loader.grub = {
    enable = lib.mkDefault true;
    device = lib.mkDefault "nodev";
    efiSupport = lib.mkDefault true;
    efiInstallAsRemovable = lib.mkDefault false;
    useOSProber = lib.mkDefault true;
    timeoutStyle = lib.mkDefault "hidden";
    extraConfig = ''
      GRUB_RECORDFAIL_TIMEOUT=0
    '';
  };

  boot.loader.efi.canTouchEfiVariables = lib.mkDefault true;
  boot.loader.timeout = lib.mkDefault 0;

  boot.plymouth = {
    enable = lib.mkDefault true;
    theme = lib.mkDefault "argentum";
    themePackages = [ argentumPlymouth ];
  };

  boot.kernelParams = [
    "quiet"
    "splash"
    "loglevel=3"
    "systemd.show_status=false"
    "rd.udev.log_level=3"
    "udev.log_priority=3"
    "vt.global_cursor_default=0"
  ];

  boot.consoleLogLevel = lib.mkDefault 0;
  boot.initrd.verbose = lib.mkDefault false;
}
