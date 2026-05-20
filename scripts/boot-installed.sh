#!/usr/bin/env bash
#
# Boot the already-installed argentumOS disk in QEMU.
#
# Pairs with test-installer.sh: that script runs Calamares against a blank
# qcow2; once the install finishes and the VM reboots, the disk holds a
# bootable argentumOS. This script boots that disk again later — no ISO, no
# Calamares, no wipe.
#
# The OVMF NVRAM copy at ./argentumOS-OVMF_VARS.fd is reused as-is. That
# file holds the EFI boot entry GRUB wrote during install ("argentumOS" →
# \EFI\NixOS-boot\grubx64.efi); recreate it and the firmware drops to the
# EFI shell instead of booting the installed system.
#
# Env vars:
#   DISK=path     disk image to boot (default: ./argentumOS.qcow2)
#   MEMORY=4G    VM RAM (default 4G)
#   SMP=4        VM vCPUs (default 4)
#   HEADLESS=1   render via VNC on :0 (port 5900) instead of GTK window
#   OVMF=path    explicit OVMF_CODE firmware (auto-detected otherwise)

set -euo pipefail
cd "$(dirname "$0")/.."

DISK="${DISK:-$PWD/argentumOS.qcow2}"
VARS="$PWD/argentumOS-OVMF_VARS.fd"
MEMORY="${MEMORY:-4G}"
SMP="${SMP:-4}"

if [[ ! -f "$DISK" ]]; then
  echo "Disk not found: $DISK" >&2
  echo "Install argentumOS first with: ./scripts/test-installer.sh" >&2
  exit 1
fi

if [[ ! -f "$VARS" ]]; then
  echo "OVMF NVRAM not found: $VARS" >&2
  echo "This file is written during the installer run and stores the EFI" >&2
  echo "boot entry GRUB created. Without it the firmware has nothing to" >&2
  echo "boot. Re-run ./scripts/test-installer.sh to recreate it." >&2
  exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null; then
  echo "qemu-system-x86_64 not found on PATH — install QEMU on the host." >&2
  exit 1
fi

ACCEL_ARG=""
CPU_ARG="-cpu qemu64"
if [[ -w /dev/kvm ]]; then
  ACCEL_ARG="-enable-kvm"
  CPU_ARG="-cpu host"
else
  echo "Warning: /dev/kvm not writable — using TCG (slow)." >&2
fi

DISPLAY_ARGS=(-display gtk)
if [[ "${HEADLESS:-0}" == "1" ]]; then
  DISPLAY_ARGS=(-display "vnc=:0" -vga virtio)
  echo "VNC available at vnc://localhost:5900"
fi

# OVMF_CODE auto-detect. Same probe order as test-vm.sh: prefer a Nix-store
# OVMF (matches the firmware used during install), then common system paths.
CODE_PATH="${OVMF:-}"
if [[ -z "$CODE_PATH" ]]; then
  shopt -s nullglob
  nix_dirs=( /nix/store/*-OVMF-*/FV /nix/store/*-OVMF-*-fd/FV )
  shopt -u nullglob
  for d in $(printf '%s\n' "${nix_dirs[@]}" | sort -u); do
    if [[ -f "$d/OVMF_CODE.fd" ]]; then
      CODE_PATH="$d/OVMF_CODE.fd"
      break
    fi
  done
fi
if [[ -z "$CODE_PATH" ]]; then
  for c in \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/edk2-ovmf/OVMF_CODE.fd \
    /usr/share/edk2/x64/OVMF_CODE.fd
  do
    if [[ -f "$c" ]]; then CODE_PATH="$c"; break; fi
  done
fi
if [[ -z "$CODE_PATH" ]]; then
  echo "OVMF_CODE firmware not found. Pass OVMF=/path/OVMF_CODE.fd." >&2
  exit 1
fi

echo "Using OVMF CODE: $CODE_PATH"
echo "Using OVMF VARS: $VARS"
echo "Booting disk:    $DISK"

exec qemu-system-x86_64 \
  -machine type=q35 \
  $ACCEL_ARG $CPU_ARG \
  -m "$MEMORY" -smp "$SMP" \
  -drive "if=pflash,format=raw,readonly=on,file=$CODE_PATH" \
  -drive "if=pflash,format=raw,file=$VARS" \
  -drive "file=$DISK,if=virtio,format=qcow2" \
  -boot order=c,menu=on \
  -netdev user,id=n0 -device virtio-net,netdev=n0 \
  -device intel-hda -device hda-duplex \
  "${DISPLAY_ARGS[@]}"
