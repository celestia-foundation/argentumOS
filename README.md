# argentumOS

A NixOS-based consumer Linux distribution focused on a polished, silent boot
experience and a heavily customized Cinnamon desktop. Apps are delivered via
Flatpak; first-class Wine integration is planned.

This repository is a **scaffold** — many subsystems (theme assets, app store
UI, Wine integration) are intentionally stubbed and will be filled in by later
work.

## Layout

```
argentumOS/
├── flake.nix                          # flake entry point + nixosConfigurations
├── Dockerfile                         # build image for non-NixOS hosts
├── docker-compose.yml                 # builder / iso / vm services
├── scripts/
│   ├── build.sh                       # nix build wrapper
│   ├── test-vm.sh                     # QEMU smoke-test runner
│   └── test-installer.sh              # boot ISO with blank disk for installer
├── modules/
│   ├── boot/default.nix               # silent GRUB + Plymouth + kernel quieting
│   ├── desktop/default.nix            # Cinnamon DE
│   ├── desktop/theme/default.nix      # GTK / icon / panel theme stubs
│   ├── apps/flatpak.nix               # Flatpak + Flathub
│   ├── wine/default.nix               # Wine module (disabled stub)
│   └── installer.nix                  # Calamares + autostart (live-ISO only)
├── installer/                         # Calamares installer (live ISO only)
│   ├── default.nix                    # installer Nix derivation
│   ├── calamares/                     # settings.conf + branding + nixos-install module
│   └── qml/                           # custom QML pages + components
└── themes/
    └── argentum-plymouth/             # Plymouth boot splash derivation
        ├── default.nix
        └── assets/
            ├── argentum.plymouth
            ├── argentum.script
            └── splash.png             # placeholder 1×1 PNG
```

## Building

### On a NixOS host

```bash
nix flake check
nixos-rebuild build --flake .#argentumOS
nix build .#argentum-plymouth
```

To build a bootable ISO, add an installer config (see
`<nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix>`) to the
flake and target `config.system.build.isoImage`.

### On a non-NixOS host (Docker)

```bash
docker compose build
docker compose run --rm builder           # full system closure
docker compose run --rm iso               # bootable ISO → ./build-output/argentumOS.iso
docker compose run --rm plymouth          # theme derivation only
docker compose --profile test up vm       # QEMU boot (needs /dev/kvm)
```

The `nix-store` named volume keeps the Nix store warm across runs, so only the
first build pays for nixpkgs evaluation. The ISO derivation lives in that
volume; `scripts/build.sh iso` copies the final `.iso` out into
`./build-output/` so it is reachable from the host.

## QEMU Testing

Uses the host's QEMU (no Nix required on host) to boot the built ISO.
Default: UEFI boot via OVMF, KVM accelerated, GTK window.

```bash
./scripts/test-vm.sh                      # UEFI + KVM + GTK
HEADLESS=1 ./scripts/test-vm.sh           # VNC at localhost:5900
BIOS=1 ./scripts/test-vm.sh               # legacy BIOS boot (no OVMF needed)
CLEAN=1 ./scripts/test-vm.sh              # drop persistent qcow2 disk first
MEMORY=8G SMP=8 ./scripts/test-vm.sh      # custom RAM / vCPU
ISO=/path/to/other.iso ./scripts/test-vm.sh
OVMF=/usr/share/OVMF/OVMF_CODE.fd ./scripts/test-vm.sh
```

Requires `qemu-system-x86_64` and (for UEFI) an `ovmf` package installed on the
host. Add yourself to the `kvm` group to avoid sudo (`sudo usermod -aG kvm
$USER` then re-login). Running under `sudo` will break GTK's X11 auth — use
`HEADLESS=1` if you must run as root.

## Replacing Placeholder Assets

- **Plymouth splash**: drop a real `splash.png` (and optional `frame-NNN.png`
  animation frames) into `themes/argentum-plymouth/assets/` and update
  `argentum.script` to cycle them.
- **GTK / icon themes**: replace `adw-gtk3` / `papirus-icon-theme` references
  in `modules/desktop/theme/default.nix` with derivations for the future
  `argentum-gtk` and `argentum-icons` packages.

## Installer

The live ISO autostarts a Calamares-based installer themed for argentumOS.

| Step          | Implementation                                    |
| ------------- | ------------------------------------------------- |
| Welcome       | Custom QML (`installer/qml/pages/Welcome.qml`)    |
| Locale        | Upstream widget, themed via `stylesheet.qss`      |
| Keyboard      | Upstream widget, themed via `stylesheet.qss`      |
| Partition     | Upstream widget, themed via `stylesheet.qss`      |
| Users         | Upstream widget, themed via `stylesheet.qss`      |
| Summary       | Custom QML (`installer/qml/pages/Summary.qml`)    |
| Install       | `installer/calamares/modules/nixos-install/main.py` runs `nixos-install` |
| Finish        | Custom QML (`installer/qml/pages/Finish.qml`)     |

The install step writes a small `configuration.nix` to the target that
imports the same `modules/` tree the live ISO ships, so the installed system
inherits every argentumOS module (boot, desktop, branding, etc.) automatically.

End-to-end test (UEFI VM, fresh 32 GB disk):

```bash
./scripts/build.sh iso
./scripts/test-installer.sh
```

See `installer/README.md` for architecture details, QML iteration without
booting a VM (`qmlscene installer/qml/main.qml`), and how to add a new
Calamares module.

## Roadmap

- **Native app store frontend** — Flatpak is the backend today; a dedicated
  GUI (with curated catalog, Wine wrappers, etc.) is a separate workstream.
- **First-class Wine integration** — DXVK/VKD3D presets, prefix management,
  Bottles-style UI, `.exe` file association handling.
- **Custom theme packages** — proper `argentum-gtk` and `argentum-icons`
  derivations replacing the current stand-ins.
