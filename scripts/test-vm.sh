#!/usr/bin/env bash
#
# Boot the built argentumOS ISO in the host's QEMU.
# No Nix on the host required — uses /usr/bin/qemu-system-x86_64 directly.
#
# Env vars:
#   ISO=path     ISO to boot (default: ./build-output/argentumOS.iso)
#   MEMORY=4G    VM RAM (default 4G)
#   SMP=4        VM vCPUs (default 4)
#   DISK_SIZE=32G  size of persistent qcow2 disk (default 32G)
#   CLEAN=1      delete the persistent qcow2 disk before booting
#   HEADLESS=1   render via VNC on :0 (port 5900) instead of GTK window
#   BIOS=1       boot legacy BIOS instead of UEFI (default UEFI via OVMF)
#   OVMF=path    explicit OVMF firmware path (auto-detected otherwise)

set -euo pipefail

cd "$(dirname "$0")/.."

ISO="${ISO:-./build-output/argentumOS.iso}"
MEMORY="${MEMORY:-4G}"
SMP="${SMP:-4}"
DISK="$PWD/argentumOS.qcow2"
DISK_SIZE="${DISK_SIZE:-32G}"

if [[ ! -f "$ISO" ]]; then
  echo "ISO not found: $ISO" >&2
  echo "Build it first: docker compose run --rm iso" >&2
  exit 1
fi

if ! command -v qemu-system-x86_64 >/dev/null; then
  echo "qemu-system-x86_64 not found on PATH — install QEMU on the host." >&2
  exit 1
fi

if [[ "${CLEAN:-0}" == "1" && -f "$DISK" ]]; then
  echo "Removing persistent disk: $DISK"
  rm -f "$DISK"
fi

if [[ ! -f "$DISK" ]]; then
  qemu-img create -f qcow2 "$DISK" "$DISK_SIZE"
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

FIRMWARE_ARGS=()
if [[ "${BIOS:-0}" != "1" ]]; then
  CODE_PATH="${OVMF:-}"
  VARS_PATH="${OVMF_VARS:-}"

  if [[ -z "$CODE_PATH" ]]; then
    shopt -s nullglob
    nix_dirs=( /nix/store/*-OVMF-*/FV /nix/store/*-OVMF-*-fd/FV )
    shopt -u nullglob
    for d in $(printf '%s\n' "${nix_dirs[@]}" | sort -u); do
      if [[ -f "$d/OVMF_CODE.fd" && -f "$d/OVMF_VARS.fd" ]]; then
        CODE_PATH="$d/OVMF_CODE.fd"
        VARS_PATH="$d/OVMF_VARS.fd"
        break
      fi
    done

    if [[ -z "$CODE_PATH" ]]; then
      pairs=(
        "/usr/share/OVMF/OVMF_CODE.fd:/usr/share/OVMF/OVMF_VARS.fd"
        "/usr/share/OVMF/OVMF_CODE_4M.fd:/usr/share/OVMF/OVMF_VARS_4M.fd"
        "/usr/share/edk2-ovmf/OVMF_CODE.fd:/usr/share/edk2-ovmf/OVMF_VARS.fd"
        "/usr/share/edk2/x64/OVMF_CODE.fd:/usr/share/edk2/x64/OVMF_VARS.fd"
      )
      for p in "${pairs[@]}"; do
        c="${p%%:*}"; v="${p##*:}"
        if [[ -f "$c" && -f "$v" ]]; then
          CODE_PATH="$c"; VARS_PATH="$v"
          break
        fi
      done
    fi
  fi

  if [[ -z "$CODE_PATH" || -z "$VARS_PATH" ]]; then
    echo "OVMF firmware (CODE+VARS) not found. Pass OVMF=/path/OVMF_CODE.fd OVMF_VARS=/path/OVMF_VARS.fd, or BIOS=1." >&2
    exit 1
  fi

  # OVMF_VARS holds NVRAM and must be writable. Stage a per-VM copy.
  VARS_COPY="$PWD/argentumOS-OVMF_VARS.fd"
  if [[ "${CLEAN:-0}" == "1" || ! -f "$VARS_COPY" ]]; then
    cp -f "$VARS_PATH" "$VARS_COPY"
    chmod u+w "$VARS_COPY"
  fi

  echo "Using OVMF CODE: $CODE_PATH"
  echo "Using OVMF VARS: $VARS_COPY (copy of $VARS_PATH)"

  FIRMWARE_ARGS=(
    -drive "if=pflash,format=raw,readonly=on,file=$CODE_PATH"
    -drive "if=pflash,format=raw,file=$VARS_COPY"
  )
fi

exec qemu-system-x86_64 \
  -machine type=q35 \
  $ACCEL_ARG $CPU_ARG \
  -m "$MEMORY" -smp "$SMP" \
  "${FIRMWARE_ARGS[@]}" \
  -drive "file=$DISK,if=virtio,format=qcow2" \
  -cdrom "$ISO" \
  -boot order=d,menu=on \
  -netdev user,id=n0 -device virtio-net,netdev=n0 \
  -device intel-hda -device hda-duplex \
  "${DISPLAY_ARGS[@]}"
