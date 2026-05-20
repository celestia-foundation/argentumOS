# Development shell for argentum-settings.
#
# Drop in with `nix-shell` from this directory. Provides the Rust toolchain,
# a C linker (`cc`), pkg-config, and every native lib GPUI needs at both
# compile time and runtime (dlopen).
#
# This mirrors the buildInputs in ./default.nix plus the rustc/cargo/cc
# toolchain (which buildRustPackage provides automatically but a dev shell
# does not).

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "argentum-settings-dev";

  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    pkg-config
    gcc           # provides `cc` linker
    cmake         # some GPUI transitive deps want it (e.g. zstd-sys)
    protobuf      # gpui uses prost for some IPC types
    perl          # openssl-sys build script
  ];

  buildInputs = with pkgs; [
    # GPUI core deps
    vulkan-loader
    libxkbcommon
    libGL
    wayland
    wayland-protocols
    fontconfig
    freetype
    expat

    # Misc native libs surfaced by Zed's crate graph
    dbus
    openssl
    alsa-lib
    libxml2
    sqlite

    # X11 stack (Zed/GPUI X11 backend) — flat names, post-deprecation
    libxcb
    libx11
    libxcursor
    libxi
    libxrandr
    libxext
    libxkbfile
    libxinerama
    libxfixes
    libxrender
    libxdamage
    libxcomposite
    libxcb-util
    libxcb-wm
    libxcb-image
    libxcb-keysyms
    libxcb-render-util
    libxcb-cursor
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    vulkan-loader
    libxkbcommon
    libGL
    wayland
    fontconfig
    freetype
    libxcb
    libx11
    libxcursor
    libxi
    libxrandr
  ]);

  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.dbus.dev}/lib/pkgconfig";

  shellHook = ''
    echo "argentum-settings dev shell ready"
    echo "  cargo check -p argentum-settings-core   # backend lib only, fast"
    echo "  cargo check                              # whole workspace incl. GPUI (slow first time)"
    echo "  cargo run -p argentum-settings -- --page network"
  '';
}
