//! Reusable input widgets — mirror of `settings-panel/.../widgets/`. Wrapping
//! `zenity` keeps the app-store binary independent of the missing GPUI
//! text-input primitive. The two crates intentionally duplicate this file
//! until a shared `argentum-ui` crate is extracted in M5.

pub mod prompt;
