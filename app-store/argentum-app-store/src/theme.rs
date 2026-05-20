//! argentumOS palette. Identical to `settings-panel/argentum-settings/src/theme.rs`;
//! kept in sync by hand for now — if it ever diverges, extract a shared crate.

/// Window background.
pub const BG: u32 = 0x1C1C1E;
/// Sidebar background.
pub const SIDEBAR: u32 = 0x161618;
/// Cards, controls, surfaces.
pub const SURFACE: u32 = 0x2C2C2E;
/// argentum gold — selection, focus rings, accents.
pub const ACCENT: u32 = 0xC8A97E;
/// Primary text.
pub const TEXT: u32 = 0xF5F5F5;
/// Secondary text.
pub const TEXT_MUTED: u32 = 0x8E8E93;

/// Accent at ~50% alpha — focus rings, in-flight underlines.
#[allow(dead_code)]
pub const ACCENT_SOFT_A: f32 = 0.5;
