{ config, lib, pkgs, ... }:

let
  cfg = config.programs.argentum-app-store;
  argentum-app-store = pkgs.callPackage ../../app-store { };
in {
  options.programs.argentum-app-store = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable the native argentumOS app store (Flathub front-end).";
    };
  };

  config = lib.mkIf cfg.enable {
    # The package ships its own .desktop entry under
    # share/applications/argentum-app-store.desktop, so no override is needed.
    environment.systemPackages = [ argentum-app-store ];
  };
}
