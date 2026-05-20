{
  description = "argentumOS — a polished, silent-boot NixOS-based consumer distribution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; config.allowUnfree = true; };

      commonModules = [
        ./modules/boot
        ./modules/desktop
        ./modules/apps/flatpak.nix
        ./modules/wine
        ./modules/branding
        ./modules/installer.nix

        ({ lib, ... }: {
          networking.hostName = "argentumOS";
          system.stateVersion = "24.11";

          nixpkgs.config.allowUnfree = true;
          nix.settings.experimental-features = [ "nix-command" "flakes" ];

          time.timeZone = lib.mkDefault "UTC";
          i18n.defaultLocale = lib.mkDefault "en_US.UTF-8";
        })
      ];

      installedExtras = [
        ({ lib, ... }: {
          users.users.argentum = {
            isNormalUser = true;
            description = "argentumOS user";
            extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
            initialPassword = "argentum";
          };

          fileSystems."/" = lib.mkDefault {
            device = "/dev/disk/by-label/nixos";
            fsType = "ext4";
          };
        })
      ];

      isoExtras = [
        "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-base.nix"
        ({ lib, ... }: {
          image.baseName = lib.mkForce "argentumOS";
          isoImage.volumeID = lib.mkForce "ARGENTUMOS";
          isoImage.makeEfiBootable = true;
          isoImage.makeUsbBootable = true;

          boot.zfs.forceImportRoot = false;

          # Silence the ISO bootloader: no menu, autoselect default entry.
          boot.loader.timeout = lib.mkForce 0;
          isoImage.appendToMenuLabel = " (argentumOS Live)";

          # Skip the LightDM greeter on the live ISO — boot straight to desktop.
          services.displayManager.autoLogin = {
            enable = true;
            user = "nixos";
          };

          # Rebrand the upstream `nixos` live user so it doesn't say "NixOS"
          # anywhere a username/description is rendered.
          users.users.nixos.description = lib.mkForce "argentumOS Live User";

          # The system hostname is set in commonModules; force it here too in
          # case installation-cd-base.nix sets a default.
          networking.hostName = lib.mkForce "argentumOS";

          # Ship the Calamares installer and autostart it on the live session.
          # The installed system keeps argentumOS.isLiveISO at its default (false)
          # so modules/installer.nix becomes a no-op there.
          argentumOS.isLiveISO = true;
        })
      ];
    in {
      nixosConfigurations.argentumOS = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs; };
        modules = commonModules ++ installedExtras;
      };

      nixosConfigurations.argentumISO = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = { inherit inputs; };
        modules = commonModules ++ isoExtras;
      };

      packages.${system} = {
        argentum-plymouth = pkgs.callPackage ./modules/boot/argentum-plymouth { };
        installer = pkgs.callPackage ./installer { };
        iso = self.nixosConfigurations.argentumISO.config.system.build.isoImage;
        toplevel = self.nixosConfigurations.argentumOS.config.system.build.toplevel;
        vm = self.nixosConfigurations.argentumOS.config.system.build.vm;
      };

      formatter.${system} = pkgs.nixpkgs-fmt;
    };
}
