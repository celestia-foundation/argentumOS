#!/usr/bin/env bash
# Boot the argentumOS live ISO in QEMU with a fresh blank disk so the
# Calamares installer can run end-to-end. Defers to scripts/test-vm.sh for the
# actual QEMU plumbing (UEFI, OVMF, KVM, port forwards).

set -euo pipefail
cd "$(dirname "$0")/.."

export CLEAN="${CLEAN:-1}"            # always boot to a blank target disk
export DISK_SIZE="${DISK_SIZE:-32G}"  # nixos-install needs real space
export MEMORY="${MEMORY:-6G}"
export ISO="${ISO:-./build-output/argentumOS.iso}"

if [ ! -f "$ISO" ]; then
  echo "ISO not found at $ISO" >&2
  echo "Build it first: ./scripts/build.sh iso" >&2
  exit 1
fi

exec ./scripts/test-vm.sh
