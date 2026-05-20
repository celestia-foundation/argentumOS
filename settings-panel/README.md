# argentum-settings

The native argentumOS system settings panel — a Rust / [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) application that replaces Cinnamon's default settings UI.

## Architecture

```
settings-panel/
├── argentum-settings/         # binary crate (UI)
│   └── src/
│       ├── main.rs            # CLI args, window opens here
│       ├── app.rs             # root layout + optimistic helper
│       ├── sidebar.rs         # animated sidebar (200ms slide)
│       ├── theme.rs           # palette constants (hardcoded — see below)
│       └── pages/             # one file per category (six total)
│           ├── mod.rs         # Page enum, PageState<T> cache, router (150ms crossfade)
│           ├── appearance.rs
│           ├── display.rs
│           ├── network.rs
│           ├── users.rs
│           ├── software.rs
│           └── system.rs
└── argentum-settings-core/    # library crate (backends; ZERO GPUI imports)
    └── src/
        ├── lib.rs             # public API, error type
        ├── display.rs         # xrandr wrapper
        ├── theme_scan.rs      # NixOS theme paths + gsettings
        ├── wallpaper_scan.rs  # /etc/backgrounds + ~/Pictures
        ├── os_release.rs      # parse /etc/os-release + hostname
        ├── system.rs          # streaming `pkexec nixos-rebuild switch --upgrade`
        └── dbus/
            ├── network.rs     # NetworkManager (zbus async)
            ├── accounts.rs    # AccountsService (zbus async)
            └── flatpak.rs     # `flatpak` CLI wrapper
```

**The two crates are split deliberately.** `argentum-settings-core` has no UI framework deps and could be reused by a CLI, a CI test rig, or a future Wayland-native panel. The binary crate is the only place GPUI types appear.

## UI philosophy

These rules are load-bearing — when in doubt during implementation, follow the rule even if it's harder to code:

1. **Direct manipulation, no Save buttons.** A change is the commit. Exceptions: destructive/expensive operations (display res, hostname, system update) use an explicit Apply.
2. **No window chrome.** No titlebar, no menubar. Sidebar + content fill the window.
3. **Calm motion.** Two animations only: sidebar slide (200ms ease-in-out), page crossfade (150ms). Nothing else animates.
4. **Hierarchy through space + color.** Four backgrounds (`bg → sidebar → surface → accent`). Borders only when space can't carry the structure.
5. **Latency honesty.** Instant (<50ms, no indicator) / Optimistic (UI flips, accent underline visible 1s min) / Confirmed (Apply + inline spinner).
6. **Errors inline, never modal.**
7. **Keyboard-reachable.** Focus rings = accent at 50% alpha.
8. **Skeletons, not spinners.** Layout is drawn on first paint with placeholder rows — never an empty pane.
9. **Density.** Apple System Settings, not Windows Control Panel. 16/24px padding.
10. **One source of truth.** Settings live in dconf/gsettings/D-Bus — the UI is a view over system state.

## State, caching, optimistic updates

Each page owns a `PageState<T>`: `Empty | Loading | Loaded { data, fetched_at } | Error`. Stale-while-revalidate: cached data is shown immediately on page revisit, and a background refresh runs if `fetched_at` exceeds the page TTL.

**TTLs** (chosen per data volatility):
- Appearance: 5 min — themes/wallpapers list, filesystem rarely changes
- Display: 10s — hotplug feel-live
- Network: 5s — active scan results
- Users: 60s
- Software: 30s
- System: 60s for os-release; hostname always re-read on entry

**Optimistic write protocol** (centralized in `app::optimistic_write`):

1. Snapshot the current value.
2. Apply locally, re-render.
3. Mark control in-flight (accent underline, 1s minimum visible).
4. `cx.spawn` the async write.
5. On success: clear in-flight marker.
6. On failure: revert via snapshot, clear marker, show inline error chip with Retry (auto-dismisses after 6s).

**Optimistic vs Confirmed:**

| Action | Mode |
|---|---|
| WiFi connect / disconnect | Optimistic |
| VPN toggle | Optimistic |
| Flatpak remote enable/disable | Optimistic |
| GTK / icon theme / font / wallpaper | Optimistic |
| Add Flatpak remote | Confirmed (network fetch) |
| Display resolution / refresh / scale | Confirmed + 15s "Keep these settings?" countdown |
| Hostname change | Confirmed (pkexec) |
| Change password | Confirmed (sheet) |
| nixos-rebuild upgrade | Confirmed (streaming log) |

## Color palette

Hardcoded in `argentum-settings/src/theme.rs`:

| Token | Hex | Use |
|---|---|---|
| `BG` | `#1C1C1E` | Window background |
| `SIDEBAR` | `#161618` | Sidebar |
| `SURFACE` | `#2C2C2E` | Cards, controls |
| `ACCENT` | `#C8A97E` | argentum gold — selection, focus, in-flight |
| `TEXT` | `#F5F5F5` | Primary text |
| `TEXT_MUTED` | `#8E8E93` | Labels, hints |

These are **not** derived from the GTK theme at runtime — argentum-settings keeps its own identity regardless of what the user picks for the rest of their desktop.

## Running in dev

```bash
cd settings-panel
cargo run -p argentum-settings -- --page network
```

`--page <name>` deep-links to a specific page on launch. Valid values: `appearance`, `display`, `network`, `users`, `software`, `system`. The binary is also re-exec'd by the `cinnamon-settings` shim with the same flag (see `modules/settings.nix`).

## Building with Nix

From the repository root:

```bash
nix build .#argentum-settings
```

**First build will fail** with a hash mismatch on the GPUI git dependency. Copy the expected hash from the error into `settings-panel/default.nix`'s `cargoLock.outputHashes` and re-run. The key shape is `"gpui-<version>" = "sha256-..."` — use the version Cargo reports for the workspace member matching the Zed `crates/gpui` `Cargo.toml`.

## Bumping the GPUI pin

GPUI is currently pinned to **Zed `v1.2.7`** (commit `deb48ceb47125223df73902d1c3b72f2f442222b`, tagged 2026-05-19). When bumping:

1. Edit `settings-panel/Cargo.toml` — change `rev = "..."` on the `gpui` workspace dep.
2. `cargo update -p gpui` to refresh `Cargo.lock`.
3. Set `cargoLock.outputHashes` entry to `lib.fakeHash`.
4. Run `nix build .#argentum-settings` — copy the new hash from the error message.
5. Expect API breakage in `argentum-settings/src/{app,sidebar,pages}.rs`. The structural shape (Page enum, PageState<T>, optimistic protocol) is GPUI-agnostic; only the call sites need adjustment.

## How to add a new settings page

1. Add a variant to `Page` in `argentum-settings/src/pages/mod.rs`. Pick a TTL via the `ttl()` match arm.
2. Add a label + icon in `Page::label()` / `Page::icon()`.
3. Create `src/pages/foo.rs` mirroring an existing page (start from `users.rs` — it's the simplest). Implement `Render`, own a `PageState<T>`, and call `spawn_refresh` on `new`.
4. Wire it into `PagesView` in `src/pages/mod.rs` (constructor + match arm in `render`).

The sidebar picks up the new entry automatically from the `Page` enum so no edits in `sidebar.rs` are required.

## Known stubs / TODOs

These are intentional and marked `// TODO:` in the source. Each has a one-line description at the stub site:

- **Wayland display backend** (`argentum-settings-core/src/display.rs`) — currently `wlr-randr` stub. argentumOS ships X11 today; revisit when Wayland lands.
- **NetworkManager WiFi enumeration** (`argentum-settings-core/src/dbus/network.rs`) — the AP-walk over `Device.Wireless` + `AccessPoint` proxies is not implemented. Page shows an inline empty-state.
- **WiFi password modal** (`argentum-settings-core/src/dbus/network.rs`) — secured networks show "Password required (coming soon)" inline.
- **Change-password sheet** (`argentum-settings-core/src/dbus/accounts.rs`) — Users page shows "Change password… (coming soon)" button.
- **Flatpak app browser** — out of scope for the settings panel; lives in the separate (planned) argentumOS App Store.
- **Text input widget** (`argentum-settings/src/pages/software.rs`) — the Add-Remote form's name/URL fields are visual placeholders. Real input requires wrapping GPUI's text-input primitive; deferred to a follow-up.

## Tests

```bash
cargo test -p argentum-settings-core
```

Covers the pure parsers (xrandr output, `/etc/os-release`, `flatpak remotes` columns). The async D-Bus paths aren't tested here — they need a live system bus and are exercised in the VM verification step.
