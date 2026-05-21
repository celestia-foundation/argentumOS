---
title: Apps
description: The native apps that ship with argentumOS, written from scratch because the alternatives made us sad.
---

# The native apps

We ship two things you don't get anywhere else. Both are written in Rust with [GPUI](https://github.com/zed-industries/zed) — Zed's UI framework — because GTK and Qt apps will simply never look like they belong on the same operating system, no matter how hard you try. We tried. It's why this section exists.

Both apps share one palette, one focus ring, one set of icons, one keyboard model. Boring on purpose.

## System Settings (`argentum-settings`)

The thing that opens when you click *Settings*. Replaces the default Cinnamon control panel.

Six categories. Network, Display, Sound, Users, Appearance, System.

It is not a frontend for `nmcli`. There are no save buttons.

- **Apply on click.** Toggle a switch, it commits immediately. The mental model is "this is what the system is now," not "this is what the system will be once you remember to press Save."
- **Confirmation only when it matters.** Change display resolution and you get a 15-second "keep these settings?" countdown, because we have made that mistake on other systems and it was not fun.
- **Streaming logs for slow stuff.** Switching kernels, rebuilding the system — you see the real output as it happens. Not a spinner pretending it knows.
- **Skeletons, not spinners.** First paint shows placeholder rows instead of a loading wheel. It feels faster because it is faster, kind of.

Under the hood: async D-Bus via [zbus](https://github.com/dbus2/zbus), tokio for the runtime, and a hard line against derived theming. The colors are fixed. The whole point is that it always looks like argentum-settings, regardless of what GTK theme the user has decided to inflict on themselves.

## App Store (`argentum-app-store`)

A first-party front-end for Flatpak / Flathub. It is **not** a Nix package browser. We picked Flatpak on purpose. Apps update without rebuilding your system at 11pm.

Pages you get:

- **Discover** — featured apps, in a real layout, not a wall of icons.
- **Categories** — browse by category, like every other store you've used.
- **Search** — instant, fuzzy enough to forgive typos.
- **Installed** — manage what you've got. Bulk update. Bulk uninstall (don't).
- **Updates** — pending updates with one click to install everything.
- **Permissions** — per-app sandbox overrides, à la Flatseal, without launching a separate app to do it.
- **Remotes** — add Flatpak remotes. Most people will never touch this. It's there.
- **Runtimes** — manage Flatpak SDKs. Prune unused ones. Reclaim disk.

Under the hood the backend wraps the `flatpak` CLI, not libflatpak's D-Bus interface. This is a deliberate choice. The CLI is the most stable surface Flatpak exposes; it's what GNOME Software and KDE Discover both fall back to in practice. The libflatpak Rust bindings are not where you want them to be yet.

Everything installs `--user`. No polkit prompts for routine operations. If you want a system-wide install, open a terminal and you've already left the spirit of the project.

## Why two custom apps and not eight

Because we picked the two surfaces a regular human touches every week: *changing settings* and *installing software*. The file manager (Nemo), the browser (Firefox), the terminal (the one you'll never need) — those are upstream, polished, and already good. Building a worse version would have been a flex, not a feature.

Wine integration is on the roadmap. Double-click a `.exe`, it runs. Ideally without summoning ancient forces.

[See the philosophy →](/philosophy)
