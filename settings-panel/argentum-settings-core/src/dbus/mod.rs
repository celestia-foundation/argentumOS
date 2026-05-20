//! D-Bus and subprocess service backends.
//!
//! - `network` — NetworkManager (system bus, async zbus)
//! - `accounts` — AccountsService (system bus, async zbus)
//! - `flatpak` — `flatpak` CLI wrapper (no D-Bus; the CLI is the stable interface)

pub mod accounts;
pub mod flatpak;
pub mod network;
