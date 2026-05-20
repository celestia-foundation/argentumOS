# Development shell for argentum-app-store.
#
# Drop in with `nix-shell` from this directory. Provides the Rust toolchain,
# a C linker (`cc`), pkg-config, and every native lib GPUI needs at both
# compile time and runtime (dlopen). Mirrors settings-panel/shell.nix.

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "argentum-app-store-dev";

  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    pkg-config
    gcc
    cmake
    protobuf
    perl
  ];

  buildInputs = with pkgs; [
    vulkan-loader
    libxkbcommon
    libGL
    wayland
    wayland-protocols
    fontconfig
    freetype
    expat

    dbus
    openssl
    alsa-lib
    libxml2
    sqlite

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

    flatpak
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
    echo "argentum-app-store dev shell ready"
    echo "  cargo check -p argentum-app-store-core   # backend only, fast"
    echo "  cargo check                               # whole workspace (slow first time)"
    echo "  cargo run -p argentum-app-store"
  '';
}
