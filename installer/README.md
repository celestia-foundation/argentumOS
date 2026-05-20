# argentumOS Installer

A Calamares-based graphical installer. Ships on the live ISO only —
`modules/installer.nix` is gated by `argentumOS.isLiveISO`, which `flake.nix`
sets to `true` exclusively for the `argentumISO` configuration.

## Architecture

```
installer/
├── default.nix                          # Nix derivation assembling the layout below
├── calamares/
│   ├── settings.conf                    # module sequence + branding selection
│   ├── branding/argentum/
│   │   ├── branding.desc                # frameless fullscreen, dark palette, slideshow
│   │   ├── stylesheet.qss               # QSS for upstream widget pages
│   │   ├── show.qml                     # sidebar slideshow (branding API 2)
│   │   └── splash.png                   # symlinked-by-source from themes/argentum-plymouth/assets
│   └── modules/nixos-install/
│       ├── module.desc
│       └── main.py                      # writes configuration.nix + runs nixos-install(8)
└── qml/
    ├── main.qml                         # standalone harness for qmlscene
    ├── pages/{Welcome,Summary,Finish}.qml
    └── components/{ArgentumButton,ArgentumInput,ArgentumProgressBar}.qml
```

## QML vs widget reality

Calamares' main window, navigation footer, and the partition / users /
keyboard / locale pages are upstream C++ / Qt widgets. We do **not** fork
Calamares, so:

| Surface                               | Implementation        | Themed how                            |
| ------------------------------------- | --------------------- | ------------------------------------- |
| Window chrome                         | Calamares C++         | `branding.desc` (frameless, fullscreen) |
| Welcome / Summary / Finish            | Our QML view-steps    | argentum palette inline                |
| Locale / Keyboard / Partition / Users | Upstream C++ widgets  | `stylesheet.qss` (QSS only)            |
| Install-phase sidebar slideshow       | Our QML               | argentum palette inline                |
| Install backend                       | Our Python module     | n/a                                    |

QSS can dark-theme the widgets convincingly, but the partition `QTreeView`
will look noticeably less polished than the QML pages. A fully-QML installer
would require a Calamares fork; outside the scope of this module.

## Module sequence

`show: welcome* → locale → keyboard → partition → users → summary*`
`exec: partition → mount → machineid → fstab → users → networkcfg → nixos-install → umount`
`show: finish*`

\* QML view-steps. The `nixos-install` exec step replaces upstream
`unpackfs` + `bootloader`.

## Iterating locally

```bash
# Build the installer derivation in isolation:
nix build .#installer -L
ls -R result/

# Preview QML pages without booting a VM (no Calamares):
qmlscene installer/qml/main.qml      # Welcome page in a frameless 1024×680 window

# Full end-to-end test in QEMU:
./scripts/build.sh iso
./scripts/test-installer.sh          # CLEAN=1, 32 GB blank disk, UEFI, KVM
```

Calamares logs on the live ISO live at `~/.cache/Calamares/session.log` and
`/var/log/calamares/` for the privileged side.

## Asset sharing with the Plymouth theme

`installer/default.nix` reads `splash.png` from
`themes/argentum-plymouth/assets/` at evaluation time. Both the Plymouth
derivation and the installer derivation consume the same source directory —
working-tree source-of-truth stays single. At runtime each derivation owns a
store path containing its own copy of `splash.png`; pure Nix cannot symlink
across derivations after build, so this is the standard pattern.

## Adding a new Calamares module

Use `installer/calamares/modules/nixos-install/` as the template:

1. Create `installer/calamares/modules/<name>/{module.desc,main.py}`.
2. Add the module name to the appropriate phase in `calamares/settings.conf`.
3. Extend `installer/default.nix` to install the new module files into
   `$out/share/calamares/modules/<name>/`.
4. Extend `modules/installer.nix` to symlink it via
   `environment.etc."calamares/modules/<name>".source = …;`.

The `nixos-install` `main.py` is the right reference for working with
`libcalamares.globalstorage`, `libcalamares.job.setprogress`, and subprocess
streaming into the Calamares log.
