#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

target="${1:-toplevel}"
mkdir -p result

# Use the `path:` flake URI rather than the bare `.` shorthand. With a `.git`
# directory present, Nix would otherwise interpret `.` as `git+file://$PWD`
# and route through libgit2 — and libgit2 rejects repositories whose
# on-disk owner UID differs from the running process UID. Inside the Docker
# builder the repo is bind-mounted from the host (UID 1000) but the build
# runs as root (UID 0), so libgit2 fails with "repository path is not owned
# by current user". `path:` reads the working tree directly off disk and
# skips libgit2 entirely; it also includes uncommitted edits, which is what
# we want for iterative ISO testing.
FLAKE="path:."

case "$target" in
  toplevel)
    nix build \
      "$FLAKE#nixosConfigurations.argentumOS.config.system.build.toplevel" \
      -o result/toplevel \
      --print-build-logs
    ;;
  iso)
    nix build \
      "$FLAKE#nixosConfigurations.argentumISO.config.system.build.isoImage" \
      -o result/iso \
      --print-build-logs
    iso_file=$(find -L result/iso/iso -name '*.iso' -type f | head -n1)
    if [[ -n "$iso_file" ]]; then
      cp -fL "$iso_file" "result/$(basename "$iso_file")"
      echo "Copied ISO to result/$(basename "$iso_file")"
    fi
    ;;
  vm)
    nix build \
      "$FLAKE#nixosConfigurations.argentumOS.config.system.build.vm" \
      -o result/vm \
      --print-build-logs
    ;;
  plymouth)
    nix build \
      "$FLAKE#argentum-plymouth" \
      -o result/plymouth \
      --print-build-logs
    ;;
  check)
    nix flake check "$FLAKE" --print-build-logs
    ;;
  *)
    echo "Unknown target: $target" >&2
    echo "Usage: $0 [toplevel|iso|vm|plymouth|check]" >&2
    exit 1
    ;;
esac

echo "Built target '$target' → result/$target"
