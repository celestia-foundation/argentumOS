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
, flatpak
, makeWrapper
}:

# argentum-app-store — the native argentumOS Flathub front-end.
#
# This package depends on GPUI, pulled from the same pinned Zed commit as
# settings-panel/. The `cargoLock` `outputHashes` entries below MUST be
# populated with the real hashes returned by the first
# `nix build .#argentum-app-store` failure — Nix prints the expected hash on
# mismatch and you paste them in here.
#
# Bumping GPUI:
#   1. Edit `app-store/Cargo.toml` and change the `rev = "..."` for gpui.
#   2. `cargo update -p gpui` to refresh `Cargo.lock`.
#   3. Set the `outputHashes` values below to `lib.fakeHash`.
#   4. Run `nix build .#argentum-app-store` — copy the new hashes from the error.

rustPlatform.buildRustPackage {
  pname = "argentum-app-store";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      # One entry per unique git source — the hash is shared across every
      # crate vendored from this Zed rev (gpui, collections, util, …).
      "collections-0.1.0" = "sha256-480HTo6kHhJ1bVUzHBxYhWI6gVCpoVyPuNNMPQ6wWbw=";
      "naga-29.0.3" = "sha256-jwPdrd2XLvK5ddEutR/39OLMh2JU3UXNWIcJKCndh+U=";
      "xim-ctext-0.3.0" = "sha256-pRT4Sz1JU9ros47/7pmIW9kosWOGMOItcnNd+VrvnpE=";
      "zed-scap-0.0.8-zed" = "sha256-BihiQHlal/eRsktyf0GI3aSWsUCW7WcICMsC2Xvb7kw=";
      "zed-font-kit-0.14.1-zed" = "sha256-KXygi0olNQi5yM8eaJVykNDtbPMDjT+cWPBF8UrtXR4=";
    };
  };

  env = { RUST_MIN_STACK = "16777216"; };

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

  postInstall = ''
    install -Dm644 argentum-app-store/resources/argentum-app-store.desktop \
      $out/share/applications/argentum-app-store.desktop
  '';

  # GPUI requires several libraries to be resolvable at runtime via dlopen.
  # Also ensure the `flatpak` CLI is on PATH for the spawned subprocesses.
  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath [
      vulkan-loader
      libxkbcommon
      libGL
      wayland
      fontconfig
    ]} $out/bin/argentum-app-store

    wrapProgram $out/bin/argentum-app-store \
      --prefix PATH : ${lib.makeBinPath [ flatpak ]}
  '';

  meta = {
    description = "Native argentumOS app store (Rust + GPUI, Flathub-backed)";
    homepage = "https://github.com/";
    license = lib.licenses.mit;
    mainProgram = "argentum-app-store";
    platforms = lib.platforms.linux;
  };
}
