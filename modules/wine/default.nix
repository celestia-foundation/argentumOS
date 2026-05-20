{ config, lib, pkgs, ... }:

let
  cfg = config.programs.argentum-wine;
in {
  options.programs.argentum-wine = {
    enable = lib.mkEnableOption "argentumOS first-class Wine integration (PLANNED)";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      wineWowPackages.stable
      winetricks
    ];
  };
}
