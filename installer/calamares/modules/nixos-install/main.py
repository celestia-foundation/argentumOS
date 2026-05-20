# argentumOS nixos-install Calamares job module.
#
# Replaces upstream `unpackfs` + `bootloader` for a NixOS target. Driven by
# the values Calamares collected in earlier `show` steps (username, hostname,
# timezone, locale, keyboard layout, partition layout) via libcalamares.globalstorage.
#
# Two responsibilities:
#   1. Copy the argentumOS modules tree (placed on the live ISO at
#      /etc/argentumOS/modules by modules/installer.nix) into the target at
#      /mnt/etc/nixos/argentumOS-modules/.
#   2. Write a minimal /mnt/etc/nixos/configuration.nix whose imports re-use
#      that modules tree, then run `nixos-generate-config --root /mnt --show-hardware-config`
#      and `nixos-install --root /mnt --no-root-passwd` to materialise the system.
#
# The generated configuration.nix structure:
#
#     { config, lib, pkgs, ... }: {
#       imports = [
#         ./hardware-configuration.nix
#         ./argentumOS-modules/installed.nix
#       ];
#       users.users.<user> = { ... };          # from Calamares
#       networking.hostName = "<host>";        # from Calamares
#       time.timeZone = "<tz>";                # from Calamares
#       i18n.defaultLocale = "<locale>";       # from Calamares
#       services.xserver.xkb.layout = "<kb>";  # from Calamares
#       system.stateVersion = "24.11";
#     }
#
# Note: argentumOS-modules/installed.nix sets argentumOS.isLiveISO = false so
# the installed system does NOT carry the Calamares autostart entry.

import os
import shutil
import subprocess
import textwrap

import libcalamares
from libcalamares.utils import target_env_call


MODULES_SRC = "/etc/argentumOS/modules"


def _gs():
    return libcalamares.globalstorage


def _target_root():
    """Where Calamares' `mount` module mounted the target filesystem.

    Calamares does not mount at /mnt: the `mount` module creates a fresh
    `/tmp/calamares-rootXXXXXX` directory, mounts the chosen root partition
    there, and publishes that path via globalStorage as `rootMountPoint`.
    Hardcoding `/mnt` made nixos-generate-config inspect an empty directory
    (no mounts under it in /proc/mounts), so the generated
    hardware-configuration.nix omitted `fileSystems."/"` and the install
    aborted with `The 'fileSystems' option does not specify your root file
    system`. Read the real path from globalStorage and fall back to /mnt
    only if the key is missing (which would itself indicate a misconfigured
    sequence and would fail loudly downstream regardless).
    """
    return _gs().value("rootMountPoint") or "/mnt"


def nix_str(s):
    """Render `s` as a Nix double-quoted string literal.

    Python's repr() / `{x!r}` formatter emits single-quoted strings, which
    are a syntax error in Nix — the generated configuration.nix failed to
    evaluate with `unexpected invalid token` on the username/description
    lines. Nix double-quoted strings need three things escaped: backslash,
    the closing double quote, and `${` (which would otherwise be parsed as
    an antiquotation). Hashed passwords from crypt(3) contain literal `$`
    characters and would trip on the last one if we used a naive wrapper.
    """
    return '"' + str(s).replace("\\", "\\\\").replace('"', '\\"').replace("${", "\\${") + '"'


def _copy_modules_tree(target):
    """Copy /etc/argentumOS/modules → <target>/etc/nixos/argentumOS-modules.

    The live ISO ships the modules tree at MODULES_SRC; we duplicate it into
    /etc/nixos on the target so the installed system can `nixos-rebuild` from
    a self-contained source tree.
    """
    dst = os.path.join(target, "etc/nixos/argentumOS-modules")
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(MODULES_SRC, dst, symlinks=True)


def _write_configuration_nix(target):
    gs = _gs()
    user_name = gs.value("username") or "argentum"
    full_name = gs.value("fullname") or "argentumOS user"
    hashed_pw = gs.value("userPasswordHashed") or "*"
    hostname  = gs.value("hostname") or "argentumOS"
    tz        = gs.value("locationTZ") or "UTC"
    locale    = gs.value("localeConf") or "en_US.UTF-8"
    if isinstance(locale, dict):
        locale = locale.get("LANG", "en_US.UTF-8")
    keyboard  = gs.value("keyboardLayout") or "us"

    body = textwrap.dedent(f"""\
        {{ config, lib, pkgs, ... }}:
        {{
          imports = [
            ./hardware-configuration.nix
            ./argentumOS-modules/installed.nix
          ];

          users.users.{user_name} = {{
            isNormalUser = true;
            description = {nix_str(full_name)};
            extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
            hashedPassword = {nix_str(hashed_pw)};
          }};

          networking.hostName = {nix_str(hostname)};
          time.timeZone = {nix_str(tz)};
          i18n.defaultLocale = {nix_str(locale)};
          services.xserver.xkb.layout = {nix_str(keyboard)};

          system.stateVersion = "24.11";
        }}
        """)

    cfg_dir = os.path.join(target, "etc/nixos")
    os.makedirs(cfg_dir, exist_ok=True)
    with open(os.path.join(cfg_dir, "configuration.nix"), "w") as fh:
        fh.write(body)


def _run(cmd, progress_msg):
    libcalamares.job.setprogress(0.0)
    libcalamares.utils.debug(f"argentumOS-install: {progress_msg}: {' '.join(cmd)}")
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    # Stream stdout to Calamares log; advance progress bar in a coarse manner.
    for line in proc.stdout:
        libcalamares.utils.debug(line.rstrip())
    proc.wait()
    if proc.returncode != 0:
        return (
            f"argentumOS install step failed",
            f"`{' '.join(cmd)}` exited with status {proc.returncode}",
        )
    return None


def run():
    """Calamares job entry point."""
    target = _target_root()
    libcalamares.utils.debug(f"argentumOS-install: target root is {target}")

    libcalamares.job.setprogress(0.05)
    _copy_modules_tree(target)

    libcalamares.job.setprogress(0.10)
    _write_configuration_nix(target)

    libcalamares.job.setprogress(0.15)
    err = _run(
        ["nixos-generate-config", "--root", target, "--show-hardware-config"],
        "generating hardware-configuration.nix",
    )
    if err:
        return err
    # nixos-generate-config with --show-hardware-config writes to stdout when
    # invoked that way in some versions; the safer call writes the file directly:
    err = _run(
        ["nixos-generate-config", "--root", target],
        "writing hardware-configuration.nix",
    )
    if err:
        return err

    libcalamares.job.setprogress(0.25)
    err = _run(
        ["nixos-install", "--root", target, "--no-root-passwd"],
        "running nixos-install",
    )
    if err:
        return err

    libcalamares.job.setprogress(1.0)
    return None
