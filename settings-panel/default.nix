{ lib
, rustPlatform
, pkg-config
, vulkan-loader
, libxkbcommon
, libGL
, wayland
, fontconfig
, freetype
, xorg
, dbus
, libxml2
, openssl
, alsa-lib
, makeWrapper
}:

# argentum-settings — the native argentumOS system settings panel.
#
# This package depends on GPUI, which is pulled from a pinned commit of
# zed-industries/zed (see settings-panel/Cargo.toml). The `cargoLock`
# `outputHashes` entry below MUST be populated with the real hash returned by
# the first `nix build .#argentum-settings` failure — Nix will print the
# expected hash on mismatch, and you paste it in here.
#
# Bumping GPUI:
#   1. Edit `settings-panel/Cargo.toml` and change the `rev = "..."` for gpui.
#   2. `cargo update -p gpui` to refresh `Cargo.lock`.
#   3. Set the `outputHashes` value below to `lib.fakeHash`.
#   4. Run `nix build .#argentum-settings` — copy the new hash from the error.

rustPlatform.buildRustPackage {
  pname = "argentum-settings";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      # TODO: replace with real hash after first nix build attempt.
      # Format: "<crate-name>-<version>" = "sha256-...";
      # Example: "gpui-0.1.0" = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    };
  };

  nativeBuildInputs = [ pkg-config makeWrapper ];

  buildInputs = [
    vulkan-loader
    libxkbcommon
    libGL
    wayland
    fontconfig
    freetype
    dbus
    libxml2
    openssl
    alsa-lib
    xorg.libxcb
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];

  # GPUI requires several libraries to be resolvable at runtime via dlopen.
  # Adding them to rpath avoids "vulkan-loader not found" failures on launch.
  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath [
      vulkan-loader
      libxkbcommon
      libGL
      wayland
      fontconfig
    ]} $out/bin/argentum-settings
  '';

  meta = {
    description = "Native argentumOS system settings panel (Rust + GPUI)";
    homepage = "https://github.com/";
    license = lib.licenses.mit;
    mainProgram = "argentum-settings";
    platforms = lib.platforms.linux;
  };
}
