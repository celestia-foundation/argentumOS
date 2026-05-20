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
import re
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


def _normalize_locale(loc):
    """Convert glibc's short codeset form to the dashed form nixpkgs accepts.

    Calamares' Locale module reports the active locale as `en_US.utf8`
    (glibc's canonical short name, no dash, lowercase). nixpkgs'
    `glibc-locales` builder parses `i18n.supportedLocales` and matches every
    entry against its SUPPORTED list, which only contains the dashed form
    `en_US.UTF-8`; the short form is rejected with
        Error: unsupported locales detected:
            en_US.utf8/UTF-8 ...
    and the whole derivation fails to build — taking the entire
    `nixos-install` step down with it. Normalising here keeps the rest of
    the installer agnostic to which form Calamares supplies.
    """
    if not loc:
        return loc
    return re.sub(r"\.utf-?8\b", ".UTF-8", loc, flags=re.IGNORECASE)


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


def _deobscure_calamares_password(s):
    """Reverse Calamares' `obscure()` pass over a stored password.

    The users module XORs each character of the typed password against
    0x1B before writing it to globalStorage under the `password` key
    (libcalamares/utils/String.cpp::obscure). The transform is symmetric
    — XORing again recovers the plaintext. Operates on Unicode code
    points, matching what Calamares does on the Qt side.
    """
    return "".join(chr(ord(c) ^ 0x1B) for c in s)


def _resolve_plaintext_password():
    """Find the password the user typed in the Calamares users page.

    Earlier revisions of this job read `userPasswordHashed` and fell back
    to `*` (locked account) when missing, leaving the installed system
    unlogginable. Then we switched to reading plaintext from
    `userPassword` → `password`, which got further: the install
    succeeded, but the password the user typed at the greeter still
    didn't work. Reason: Calamares' users module XORs the password with
    0x1B before storing it under `password`
    (`libcalamares/utils/String.cpp::obscure` — symmetric). We were
    handing chpasswd the *obscured* string, so /etc/shadow ended up with
    a hash of the cipher, not the cleartext.

    Order of preference now:
      1. `userPassword` — newer Calamares may expose plaintext here.
         Logged as a GS warning today because our build doesn't set it,
         but cheap to keep for forward compat.
      2. `password` — present on every Calamares build, XOR-obscured.
         Deobscure before returning.

    Returns None when neither key is set so the caller can fail loudly
    instead of installing a locked system.
    """
    gs = _gs()
    pw = gs.value("userPassword")
    if pw:
        libcalamares.utils.debug("argentumOS-install: password sourced from globalStorage[userPassword] (plaintext)")
        return pw
    obscured = gs.value("password")
    if obscured:
        libcalamares.utils.debug("argentumOS-install: password sourced from globalStorage[password] (obscured); deobscuring")
        return _deobscure_calamares_password(obscured)
    return None


def _write_configuration_nix(target):
    gs = _gs()
    user_name = gs.value("username") or "argentum"
    full_name = gs.value("fullname") or "argentumOS user"
    hostname  = gs.value("hostname") or "argentumOS"
    tz        = gs.value("locationTZ") or "UTC"
    locale    = gs.value("localeConf") or "en_US.UTF-8"
    if isinstance(locale, dict):
        locale = locale.get("LANG", "en_US.UTF-8")
    locale    = _normalize_locale(locale)
    keyboard  = gs.value("keyboardLayout") or "us"

    # The password is NOT declared here. `nixos-install` runs activation
    # which materialises the user with an empty/locked shadow entry
    # (NixOS' default when `hashedPassword` is unset and `mutableUsers`
    # is true, which it is by default). After activation we chroot into
    # the target and use chpasswd to write the real password — see
    # `_set_user_password`. Keeping it out of configuration.nix means the
    # plaintext never lands in a world-readable file in /etc/nixos and
    # later `nixos-rebuild`s don't blow the password away on activation.
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


def _set_user_password(install_root, user_name, plaintext):
    """Set <user_name>'s password on the freshly-installed target.

    Uses `nixos-enter --root <target> -- chpasswd`. nixos-enter is shipped
    by `nixos-install-tools` on every NixOS installer ISO and does the
    things plain `chroot` does not:

      - bind-mounts /proc, /sys, /dev into the target,
      - mounts a tmpfs at <target>/run and creates the
        /run/current-system → /nix/var/nix/profiles/system symlink, which
        is what makes /run/current-system/sw/bin/ resolvable,
      - execs the command through the target's bash with PATH set to the
        system profile — so plain `chpasswd` is found.

    A bare `chroot <target> chpasswd` exits with status 127 (the previous
    failure mode) because /run/current-system doesn't exist yet inside
    the new root, leaving chpasswd unfindable.

    chpasswd reads `user:plaintext` on stdin and writes /etc/shadow
    directly, bypassing PAM's password-quality checks — what the user
    typed in Calamares is what we set, verbatim, with the target's own
    crypt configuration (so the algorithm is guaranteed to match what
    PAM later reads back at the greeter).
    """
    libcalamares.utils.debug(f"argentumOS-install: setting password for {user_name} via nixos-enter + chpasswd")
    proc = subprocess.run(
        ["nixos-enter", "--root", install_root, "--", "chpasswd"],
        input=f"{user_name}:{plaintext}\n",
        text=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
    )
    if proc.stdout:
        for line in proc.stdout.splitlines():
            libcalamares.utils.debug(f"nixos-enter: {line}")
    if proc.returncode != 0:
        return (
            "argentumOS install step failed",
            f"nixos-enter chpasswd in the target exited with status {proc.returncode}",
        )
    return None


def _bind_to_nontmp(target):
    """Expose `target` and every submount under it at /run/calamares-target.

    Nix refuses to operate when an ancestor of its store is world-writable —
    LocalStore aborts with `Path "/tmp" is world-writable or a symlink.
    That's not allowed for security.` Calamares' `mount` module roots the
    target at /tmp/calamares-rootXXXX (and /tmp is mode 1777), so handing
    that path to `nixos-install --root` fails the check the moment nix-build
    spins up.

    /run on NixOS is mode 0755 root:root, so a bind mount of the target into
    /run/calamares-target gives nix the same filesystem via a path with no
    world-writable ancestor.

    The bind must be **recursive** (`--rbind`): Calamares' partition+mount
    sequence mounts the EFI System Partition as a submount at
    `<target>/boot`, and a plain `--bind` clones only the top-level
    filesystem — so `/run/calamares-target/boot` would be the empty
    directory on the root partition, not the ESP. When nixos-install's
    bootloader step then runs `grub-install --efi-directory=/boot` inside
    the chroot, GRUB inspects the directory, finds no FAT32, and fails with
    `/boot doesn't look like an EFI partition.` `--rbind` carries every
    submount along so the chroot sees the same layout Calamares set up.

    Callers must `umount -R` the bind in a finally so Calamares' later
    `umount` step still tears the real mounts down cleanly.
    """
    bind_dir = "/run/calamares-target"
    os.makedirs(bind_dir, exist_ok=True)
    # Drop any stale (recursive) bind from a previous attempt before binding.
    subprocess.run(
        ["umount", "--recursive", "--lazy", bind_dir],
        check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    subprocess.run(["mount", "--rbind", target, bind_dir], check=True)
    return bind_dir


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
    install_root = _bind_to_nontmp(target)
    try:
        err = _run(
            ["nixos-install", "--root", install_root, "--no-root-passwd"],
            "running nixos-install",
        )
        if err is None:
            # Set the password on the now-materialised target before we drop
            # the bind. The chroot needs the rbinded path so /proc, /sys,
            # /dev and the ESP submount stay visible to chpasswd's PAM stack.
            user_name = _gs().value("username") or "argentum"
            plaintext = _resolve_plaintext_password()
            if plaintext is None:
                err = (
                    "argentumOS install step failed",
                    "Calamares did not record a plaintext password in "
                    "globalStorage (checked userPassword, password); the "
                    "installed system would have a locked account.",
                )
            else:
                err = _set_user_password(install_root, user_name, plaintext)
    finally:
        # `-R` mirrors the recursive bind above; without it the ESP submount
        # at <bind>/boot is left behind and Calamares' subsequent `umount`
        # step on the real target sees the partition as still busy.
        subprocess.run(
            ["umount", "--recursive", install_root],
            check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
    if err:
        return err

    libcalamares.job.setprogress(1.0)
    return None
