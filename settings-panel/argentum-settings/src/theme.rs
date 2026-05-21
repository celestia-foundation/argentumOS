//! argentumOS palette. Hardcoded, **not** derived from the GTK theme — the
//! settings panel keeps its own identity regardless of the system theme the
//! user picks for everything else.
//!
//! All values are sRGB hex; pass them to GPUI's color constructors at use sites.

/// Window background.
pub const BG: u32 = 0x1C1C1E;
/// Sidebar background (slightly darker than `BG`).
pub const SIDEBAR: u32 = 0x161618;
/// Cards, controls, surfaces.
pub const SURFACE: u32 = 0x2C2C2E;
/// argentum gold — selection, focus rings, accents.
pub const ACCENT: u32 = 0xC8A97E;
/// Primary text.
pub const TEXT: u32 = 0xF5F5F5;
/// Secondary text (labels, hints).
pub const TEXT_MUTED: u32 = 0x8E8E93;

/// Accent at ~50% alpha — focus rings, in-flight underlines.
#[allow(dead_code)] // referenced by focus-ring code that's coming in a follow-up
pub const ACCENT_SOFT_A: f32 = 0.5;

/// Input field background — slightly darker than `SURFACE` for visual depth.
#[allow(dead_code)]
pub const INPUT_BG: u32 = 0x141416;
/// Input field 1px border in resting state.
#[allow(dead_code)]
pub const INPUT_BORDER: u32 = 0x3A3A3C;
/// Input field border on focus / active state.
#[allow(dead_code)]
pub const INPUT_FOCUS: u32 = ACCENT;
