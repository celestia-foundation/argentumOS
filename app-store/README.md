# argentum-app-store

The native argentumOS app store: a Rust/GPUI front-end for Flatpak/Flathub.

Sibling submodule to `settings-panel/`, sharing its architectural pattern:

```
app-store/
  argentum-app-store/         # binary crate, GPUI UI
  argentum-app-store-core/    # library crate, backend (flatpak CLI, AppStream, Flathub API)
  default.nix                 # rustPlatform.buildRustPackage
  shell.nix                   # nix-shell dev environment
```

## Pages

`Discover` · `Categories` · `Search` · `Installed` · `Updates` · `Permissions` · `Remotes` · `Runtimes`

- **Discover / Categories / Search** drill into per-app detail pages.
- **Installed / Updates** list installed apps with size/version + bulk update.
- **Permissions** is a Flatseal-style per-app sandbox override editor.
- **Remotes** mirrors and extends the remote management in `argentum-settings`.
- **Runtimes** lists installed runtimes/SDKs with a "prune unused" action.

All installs go to `--user` scope by default — no polkit prompt for routine install/uninstall/update.

## Architecture

- **Backend (core crate):** wraps the `flatpak` CLI as subprocess calls, with an `on_runtime` tokio bridge so smol-based GPUI can `.await` tokio I/O. No GPUI imports.
- **Metadata pipeline:** local AppStream XML (`~/.local/share/flatpak/appstream/<remote>/<arch>/active/appstream.xml.gz`) for fast offline catalog; Flathub HTTP API (`flathub.org/api/v2`) for rich detail-page fields (screenshots, verified badge, install counts). Both cached on disk.
- **UI:** same `PageState<T>` stale-while-revalidate cache and optimistic-update protocol as `argentum-settings`. Same hardcoded palette tokens; no live theming.
- **Streaming progress:** install/uninstall/update/prune use the same `mpsc<LogLine>` streaming pattern as `argentum-settings-core::system::run_upgrade`.

## Dev loop

```
cd app-store
nix-shell
cargo run -p argentum-app-store
```

## Build with Nix

```
nix build .#argentum-app-store
```

First build will fail at `cargoLock.outputHashes`; copy the hashes Nix prints into `default.nix`.

## Bumping the GPUI pin

1. Edit `Cargo.toml` workspace deps `rev = "..."`.
2. `cargo update -p gpui`.
3. Set `outputHashes` values in `default.nix` to `lib.fakeHash`.
4. `nix build .#argentum-app-store` — copy real hashes from the error.

## Why CLI, not libflatpak D-Bus

The `flatpak` CLI is the stable interface — it survives libflatpak ABI churn, has predictable output formats with `--columns=`, and matches what every other front-end (GNOME Software, KDE Discover) eventually falls back to for transactional operations. The `org.freedesktop.Flatpak` D-Bus surface has rough async ergonomics from Rust today; revisit if libflatpak Rust bindings mature.
